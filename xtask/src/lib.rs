use chrono::{NaiveDate, Utc};
use nexrad_model::meta::registry;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Display};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const NOAA_RADAR_SITES_URL: &str = "https://opengeo.ncep.noaa.gov/geoserver/nws/ows?service=WFS&version=1.0.0&request=GetFeature&typeName=nws%3Aradar_sites&outputFormat=csv";

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

const SNAPSHOT_SCHEMA_VERSION: u32 = 1;
const REGISTRY_COORDINATE_TOLERANCE: f64 = 0.01;
const SOURCE_COORDINATE_TOLERANCE: f64 = 0.0001;
const SOURCE_ELEVATION_TOLERANCE_METERS: f64 = 1.0;
const EXCLUDED_SITE_IDS: &[&str] = &["KBIX", "KCRI", "KLIX", "KOUN"];

#[derive(Debug, Clone, PartialEq)]
pub struct RadarSiteSnapshot {
    pub schema_version: u32,
    pub last_verified: String,
    pub source: String,
    pub sites: Vec<RadarSite>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RadarSite {
    pub id: String,
    pub registry_name: String,
    pub source_name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation_meters: f64,
}

#[derive(Debug)]
struct MessageError(String);

impl Display for MessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for MessageError {}

fn error(message: impl Into<String>) -> Box<dyn Error + Send + Sync> {
    Box::new(MessageError(message.into()))
}

fn with_context<T, E: Display>(
    result: std::result::Result<T, E>,
    context: impl Display,
) -> Result<T> {
    result.map_err(|source| error(format!("{context}: {source}")))
}

pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask must be in the workspace root")
        .to_path_buf()
}

pub fn snapshot_path() -> PathBuf {
    workspace_root().join("nexrad-model/data/operational-radar-sites.csv")
}

pub fn candidate_path() -> PathBuf {
    workspace_root().join("target/radar-sites-current.csv")
}

fn parse_csv(contents: &str) -> Result<Vec<Vec<String>>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut characters = contents.chars().peekable();
    let mut in_quotes = false;
    let mut closed_quote = false;

    while let Some(character) = characters.next() {
        if in_quotes {
            if character == '"' {
                if characters.peek() == Some(&'"') {
                    characters.next();
                    field.push('"');
                } else {
                    in_quotes = false;
                    closed_quote = true;
                }
            } else {
                field.push(character);
            }
            continue;
        }

        match character {
            '"' if field.is_empty() && !closed_quote => in_quotes = true,
            ',' => {
                row.push(std::mem::take(&mut field));
                closed_quote = false;
            }
            '\n' => {
                row.push(std::mem::take(&mut field));
                if row.iter().any(|value| !value.is_empty()) {
                    rows.push(std::mem::take(&mut row));
                } else {
                    row.clear();
                }
                closed_quote = false;
            }
            '\r' if characters.peek() == Some(&'\n') => {}
            _ if closed_quote => {
                return Err(error("unexpected character after closing CSV quote"));
            }
            _ => field.push(character),
        }
    }

    if in_quotes {
        return Err(error("unterminated quoted CSV field"));
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    Ok(rows)
}

fn csv_column(headers: &[String], name: &str) -> Result<usize> {
    headers
        .iter()
        .position(|header| header == name)
        .ok_or_else(|| error(format!("CSV is missing required {name:?} column")))
}

fn csv_value<'a>(row: &'a [String], column: usize, name: &str) -> Result<&'a str> {
    row.get(column)
        .map(String::as_str)
        .ok_or_else(|| error(format!("CSV row is missing {name:?} value")))
}

fn parse_number(value: &str, field: &str, id: &str) -> Result<f64> {
    let number: f64 = with_context(value.parse(), format!("parse {field} for radar site {id}"))?;
    if !number.is_finite() {
        return Err(error(format!(
            "{field} for radar site {id} must be finite, got {value:?}"
        )));
    }
    if (field == "latitude" && !(-90.0..=90.0).contains(&number))
        || (field == "longitude" && !(-180.0..=180.0).contains(&number))
    {
        return Err(error(format!(
            "{field} for radar site {id} is out of range: {value}"
        )));
    }
    Ok(number)
}

pub fn load_snapshot(path: &Path) -> Result<RadarSiteSnapshot> {
    let contents = with_context(
        fs::read_to_string(path),
        format!("read radar site snapshot {}", path.display()),
    )?;
    let rows = with_context(
        parse_csv(&contents),
        format!("parse radar site snapshot {}", path.display()),
    )?;
    if rows.len() < 4 {
        return Err(error("radar site snapshot is missing metadata or headers"));
    }

    let metadata = |index: usize, key: &str| -> Result<&str> {
        let row = &rows[index];
        if row.first().map(String::as_str) != Some(key) || row.len() != 2 {
            return Err(error(format!(
                "radar site snapshot row {} must be {key},<value>",
                index + 1
            )));
        }
        Ok(&row[1])
    };
    let schema_version = with_context(
        metadata(0, "schema_version")?.parse::<u32>(),
        "parse snapshot schema_version",
    )?;
    if schema_version != SNAPSHOT_SCHEMA_VERSION {
        return Err(error(format!(
            "unsupported radar site snapshot schema version {schema_version}"
        )));
    }
    let last_verified = metadata(1, "last_verified")?.to_string();
    with_context(
        NaiveDate::parse_from_str(&last_verified, "%Y-%m-%d"),
        "parse snapshot last_verified as YYYY-MM-DD",
    )?;
    let source = metadata(2, "source")?.to_string();

    let headers = &rows[3];
    let id_column = csv_column(headers, "id")?;
    let registry_name_column = csv_column(headers, "registry_name")?;
    let source_name_column = csv_column(headers, "source_name")?;
    let latitude_column = csv_column(headers, "latitude")?;
    let longitude_column = csv_column(headers, "longitude")?;
    let elevation_column = csv_column(headers, "elevation_meters")?;
    let mut sites = Vec::new();
    let mut identifiers = BTreeSet::new();

    for row in &rows[4..] {
        let id = csv_value(row, id_column, "id")?.to_string();
        if !identifiers.insert(id.clone()) {
            return Err(error(format!(
                "radar site snapshot contains duplicate identifier {id}"
            )));
        }
        sites.push(RadarSite {
            registry_name: csv_value(row, registry_name_column, "registry_name")?.to_string(),
            source_name: csv_value(row, source_name_column, "source_name")?.to_string(),
            latitude: parse_number(
                csv_value(row, latitude_column, "latitude")?,
                "latitude",
                &id,
            )?,
            longitude: parse_number(
                csv_value(row, longitude_column, "longitude")?,
                "longitude",
                &id,
            )?,
            elevation_meters: parse_number(
                csv_value(row, elevation_column, "elevation_meters")?,
                "elevation",
                &id,
            )?,
            id,
        });
    }

    Ok(RadarSiteSnapshot {
        schema_version,
        last_verified,
        source,
        sites,
    })
}

pub async fn fetch_operational_sites() -> Result<Vec<RadarSite>> {
    let client = reqwest::Client::builder()
        .user_agent("nexrad-xtask radar-site-registry-audit")
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build();
    let client = with_context(client, "build NOAA radar site client")?;
    let response = with_context(
        client.get(NOAA_RADAR_SITES_URL).send().await,
        "download NOAA radar site CSV",
    )?;
    let response = with_context(
        response.error_for_status(),
        "NOAA radar site CSV returned an error",
    )?;
    let body = with_context(response.text().await, "read NOAA radar site CSV")?;
    parse_operational_sites(&body)
}

fn parse_operational_sites(body: &str) -> Result<Vec<RadarSite>> {
    let rows = with_context(parse_csv(body), "parse NOAA radar site CSV")?;
    let headers = rows
        .first()
        .ok_or_else(|| error("NOAA radar site CSV is empty"))?;
    let id_column = csv_column(headers, "rda_id")?;
    let name_column = csv_column(headers, "name")?;
    let latitude_column = csv_column(headers, "lat")?;
    let longitude_column = csv_column(headers, "lon")?;
    let elevation_column = csv_column(headers, "elevmeter")?;

    let registry_names: BTreeMap<_, _> = registry::sites()
        .iter()
        .map(|site| (site.id, site.city))
        .collect();
    let mut sites = BTreeMap::new();

    for row in &rows[1..] {
        let id = csv_value(row, id_column, "rda_id")?
            .trim()
            .to_ascii_uppercase();
        if !is_operational_us_wsr88d(&id) {
            continue;
        }
        let source_name = csv_value(row, name_column, "name")?.trim();

        let site = RadarSite {
            registry_name: registry_names
                .get(id.as_str())
                .copied()
                .unwrap_or(source_name)
                .to_string(),
            source_name: source_name.to_string(),
            latitude: parse_number(csv_value(row, latitude_column, "lat")?, "latitude", &id)?,
            longitude: parse_number(csv_value(row, longitude_column, "lon")?, "longitude", &id)?,
            elevation_meters: parse_number(
                csv_value(row, elevation_column, "elevmeter")?,
                "elevation",
                &id,
            )?,
            id: id.clone(),
        };

        // NOAA's layer can list a site more than once (KHDC appears seven times). Identical rows
        // are collapsed; rows that disagree are an error.
        match sites.entry(id.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(site);
            }
            Entry::Occupied(entry) if *entry.get() != site => {
                return Err(error(format!(
                    "NOAA radar site source contains conflicting rows for identifier {id}"
                )));
            }
            Entry::Occupied(_) => {}
        }
    }

    let sites: Vec<_> = sites.into_values().collect();
    if sites.is_empty() {
        return Err(error(
            "NOAA radar site source contained no operational WSR-88D sites",
        ));
    }
    Ok(sites)
}

fn is_operational_us_wsr88d(id: &str) -> bool {
    if id.len() != 4 || EXCLUDED_SITE_IDS.contains(&id) {
        return false;
    }
    id.starts_with('K') || id.starts_with('P') || id == "TJUA"
}

pub fn compare_registry(snapshot: &RadarSiteSnapshot) -> Vec<String> {
    let expected: BTreeMap<_, _> = snapshot
        .sites
        .iter()
        .map(|site| (site.id.as_str(), site))
        .collect();
    let actual: BTreeMap<_, _> = registry::sites()
        .iter()
        .map(|site| (site.id, site))
        .collect();
    let mut differences = Vec::new();

    append_membership_differences(
        expected.keys().copied().collect(),
        actual.keys().copied().collect(),
        "registry is missing operational site",
        "registry contains unexpected site",
        &mut differences,
    );

    for (id, expected_site) in expected {
        let Some(actual_site) = actual.get(id) else {
            continue;
        };
        if actual_site.city != expected_site.registry_name {
            differences.push(format!(
                "{id} registry name changed: snapshot {:?}, registry {:?}",
                expected_site.registry_name, actual_site.city
            ));
        }
        if (actual_site.latitude as f64 - expected_site.latitude).abs()
            > REGISTRY_COORDINATE_TOLERANCE
        {
            differences.push(format!(
                "{id} latitude differs: snapshot {:.5}, registry {:.5}",
                expected_site.latitude, actual_site.latitude
            ));
        }
        if (actual_site.longitude as f64 - expected_site.longitude).abs()
            > REGISTRY_COORDINATE_TOLERANCE
        {
            differences.push(format!(
                "{id} longitude differs: snapshot {:.5}, registry {:.5}",
                expected_site.longitude, actual_site.longitude
            ));
        }
    }
    differences
}

pub fn compare_source(snapshot: &RadarSiteSnapshot, live_sites: &[RadarSite]) -> Vec<String> {
    let expected: BTreeMap<_, _> = snapshot
        .sites
        .iter()
        .map(|site| (site.id.as_str(), site))
        .collect();
    let actual: BTreeMap<_, _> = live_sites
        .iter()
        .map(|site| (site.id.as_str(), site))
        .collect();
    let mut differences = Vec::new();

    append_membership_differences(
        actual.keys().copied().collect(),
        expected.keys().copied().collect(),
        "NOAA added operational site",
        "NOAA no longer lists site",
        &mut differences,
    );

    for (id, expected_site) in expected {
        let Some(actual_site) = actual.get(id) else {
            continue;
        };
        if expected_site.source_name != actual_site.source_name {
            differences.push(format!(
                "{id} NOAA name changed: snapshot {:?}, current {:?}",
                expected_site.source_name, actual_site.source_name
            ));
        }
        append_numeric_difference(
            id,
            "latitude",
            expected_site.latitude,
            actual_site.latitude,
            SOURCE_COORDINATE_TOLERANCE,
            &mut differences,
        );
        append_numeric_difference(
            id,
            "longitude",
            expected_site.longitude,
            actual_site.longitude,
            SOURCE_COORDINATE_TOLERANCE,
            &mut differences,
        );
        append_numeric_difference(
            id,
            "elevation meters",
            expected_site.elevation_meters,
            actual_site.elevation_meters,
            SOURCE_ELEVATION_TOLERANCE_METERS,
            &mut differences,
        );
    }
    differences
}

fn append_membership_differences(
    expected_ids: BTreeSet<&str>,
    actual_ids: BTreeSet<&str>,
    missing_message: &str,
    unexpected_message: &str,
    differences: &mut Vec<String>,
) {
    for id in expected_ids.difference(&actual_ids) {
        differences.push(format!("{missing_message} {id}"));
    }
    for id in actual_ids.difference(&expected_ids) {
        differences.push(format!("{unexpected_message} {id}"));
    }
}

fn append_numeric_difference(
    id: &str,
    field: &str,
    expected: f64,
    actual: f64,
    tolerance: f64,
    differences: &mut Vec<String>,
) {
    if (expected - actual).abs() > tolerance {
        differences.push(format!(
            "{id} NOAA {field} changed: snapshot {expected:.5}, current {actual:.5}"
        ));
    }
}

pub fn snapshot_from_live(live_sites: Vec<RadarSite>, today: NaiveDate) -> RadarSiteSnapshot {
    RadarSiteSnapshot {
        schema_version: SNAPSHOT_SCHEMA_VERSION,
        last_verified: today.format("%Y-%m-%d").to_string(),
        source: NOAA_RADAR_SITES_URL.to_string(),
        sites: live_sites,
    }
}

fn append_csv_row(output: &mut String, values: &[&str]) {
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        if value.contains([',', '"', '\r', '\n']) {
            output.push('"');
            output.push_str(&value.replace('"', "\"\""));
            output.push('"');
        } else {
            output.push_str(value);
        }
    }
    output.push('\n');
}

pub fn write_snapshot(snapshot: &RadarSiteSnapshot, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        with_context(
            fs::create_dir_all(parent),
            format!("create snapshot directory {}", parent.display()),
        )?;
    }

    let mut serialized = String::new();
    let schema_version = snapshot.schema_version.to_string();
    append_csv_row(&mut serialized, &["schema_version", &schema_version]);
    append_csv_row(&mut serialized, &["last_verified", &snapshot.last_verified]);
    append_csv_row(&mut serialized, &["source", &snapshot.source]);
    append_csv_row(
        &mut serialized,
        &[
            "id",
            "registry_name",
            "source_name",
            "latitude",
            "longitude",
            "elevation_meters",
        ],
    );
    for site in &snapshot.sites {
        let latitude = site.latitude.to_string();
        let longitude = site.longitude.to_string();
        let elevation = site.elevation_meters.to_string();
        append_csv_row(
            &mut serialized,
            &[
                &site.id,
                &site.registry_name,
                &site.source_name,
                &latitude,
                &longitude,
                &elevation,
            ],
        );
    }
    with_context(
        fs::write(path, serialized),
        format!("write snapshot {}", path.display()),
    )
}

/// Compares the Rust registry with the checked-in snapshot without using the network.
pub fn check_registry_offline() -> Result<RadarSiteSnapshot> {
    let snapshot = load_snapshot(&snapshot_path())?;
    let registry_differences = compare_registry(&snapshot);
    if !registry_differences.is_empty() {
        return Err(error(format_differences(
            "local registry differs from the checked-in radar site snapshot",
            &registry_differences,
        )));
    }
    Ok(snapshot)
}

/// Compares the checked-in snapshot with NOAA's current catalog.
///
/// This requires network access and is intended for the scheduled audit workflow and manual use,
/// not for the default test suite.
pub async fn check_against_noaa() -> Result<()> {
    let snapshot = check_registry_offline()?;
    let live_sites = fetch_operational_sites().await?;
    let source_differences = compare_source(&snapshot, &live_sites);
    if source_differences.is_empty() {
        println!(
            "OK: Rust registry and checked-in snapshot match ({} sites).",
            snapshot.sites.len()
        );
        println!(
            "OK: live NOAA catalog matches the snapshot ({} sites; snapshot last updated {}).",
            live_sites.len(),
            snapshot.last_verified
        );
        return Ok(());
    }

    let today = Utc::now().date_naive();

    let candidate = snapshot_from_live(live_sites, today);
    let candidate_path = candidate_path();
    write_snapshot(&candidate, &candidate_path)?;
    Err(error(format!(
        "{}\n\nCandidate snapshot written to {}.\nRun: cargo run -p xtask -- update-radar-sites",
        format_differences(
            "NOAA operational radar site data differs from the checked-in snapshot",
            &source_differences
        ),
        candidate_path.display()
    )))
}

pub async fn update_radar_sites() -> Result<()> {
    let today = Utc::now().date_naive();
    let snapshot = snapshot_from_live(fetch_operational_sites().await?, today);
    let path = snapshot_path();
    write_snapshot(&snapshot, &path)?;
    println!("Updated {}", path.display());

    let differences = compare_registry(&snapshot);
    if !differences.is_empty() {
        return Err(error(format_differences(
            "snapshot updated, but the Rust registry still needs changes",
            &differences,
        )));
    }
    println!(
        "OK: Rust registry matches the updated NOAA snapshot ({} sites).",
        snapshot.sites.len()
    );
    Ok(())
}

pub fn format_differences(title: &str, differences: &[String]) -> String {
    let details = differences
        .iter()
        .map(|difference| format!("  - {difference}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{title}:\n{details}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noaa_csv_rejects_invalid_numbers() {
        for (latitude, longitude, elevation) in [
            ("NaN", "-20", "100"),
            ("10", "NaN", "100"),
            ("10", "-20", "NaN"),
            ("inf", "-20", "100"),
            ("10", "-inf", "100"),
            ("91", "-20", "100"),
            ("10", "-181", "100"),
        ] {
            let csv = format!(
                "rda_id,name,lat,lon,elevmeter\nKAAA,Test,{latitude},{longitude},{elevation}\n"
            );
            assert!(parse_operational_sites(&csv).is_err(), "accepted {csv}");
        }
    }

    #[test]
    fn noaa_csv_validates_schema_duplicates_and_network_membership() {
        let headers = "rda_id,name,lat,lon,elevmeter\n";
        let row = "KAAA,Test,10,-20,100\n";
        let sites = parse_operational_sites(&format!(
            "{headers}{row}KLIX,Retired,30,-90,10\nRKJK,Overseas,30,120,10\n"
        ))
        .unwrap();
        assert_eq!(sites, vec![site("KAAA", "Test", 10.0, -20.0)]);
        assert!(parse_operational_sites(&format!("{headers}{row}KAAA,Test,10,-21,100\n")).is_err());
        assert!(parse_operational_sites(headers).is_err());
        assert!(parse_operational_sites("rda_id,name\nKAAA,Test\n").is_err());
    }

    #[test]
    fn noaa_csv_collapses_identical_duplicate_rows() {
        let csv = "rda_id,name,lat,lon,elevmeter\nKAAA,Test,10,-20,100\nKAAA,Test,10,-20,100\n";
        let sites = parse_operational_sites(csv).unwrap();
        assert_eq!(sites, vec![site("KAAA", "Test", 10.0, -20.0)]);
    }

    fn site(id: &str, name: &str, latitude: f64, longitude: f64) -> RadarSite {
        RadarSite {
            id: id.to_string(),
            registry_name: name.to_string(),
            source_name: name.to_string(),
            latitude,
            longitude,
            elevation_meters: 100.0,
        }
    }

    #[test]
    fn source_comparison_reports_membership_name_and_coordinate_changes() {
        let snapshot = RadarSiteSnapshot {
            schema_version: 1,
            last_verified: "2026-01-01".to_string(),
            source: NOAA_RADAR_SITES_URL.to_string(),
            sites: vec![
                site("KAAA", "Old Name", 10.0, -20.0),
                site("KOLD", "Retired", 30.0, -40.0),
            ],
        };
        let live = vec![
            site("KAAA", "New Name", 10.1, -20.0),
            site("KNEW", "New Site", 50.0, -60.0),
        ];

        let differences = compare_source(&snapshot, &live).join("\n");
        assert!(differences.contains("NOAA added operational site KNEW"));
        assert!(differences.contains("NOAA no longer lists site KOLD"));
        assert!(differences.contains("KAAA NOAA name changed"));
        assert!(differences.contains("KAAA NOAA latitude changed"));
    }

    #[test]
    fn operational_filter_excludes_non_network_and_overseas_sites() {
        for id in ["KDOX", "KLGX", "PAPD", "TJUA"] {
            assert!(is_operational_us_wsr88d(id));
        }
        for id in ["KBIX", "KCRI", "KLIX", "KOUN", "RKJK", "TADW"] {
            assert!(!is_operational_us_wsr88d(id));
        }
    }

    #[test]
    fn csv_parser_handles_quoted_commas_quotes_and_newlines() {
        let rows = parse_csv(
            "id,name\r\nKAAA,Plain\r\nKBBB,\"Comma, Name\"\r\nKCCC,\"Line 1\nLine \"\"2\"\"\"\r\n",
        )
        .unwrap();

        assert_eq!(rows[0], ["id", "name"]);
        assert_eq!(rows[1], ["KAAA", "Plain"]);
        assert_eq!(rows[2], ["KBBB", "Comma, Name"]);
        assert_eq!(rows[3], ["KCCC", "Line 1\nLine \"2\""]);
    }
}
