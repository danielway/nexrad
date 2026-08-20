use anyhow::{bail, Context, Result};
use chrono::{NaiveDate, Utc};
use nexrad_model::meta::registry;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const NOAA_RADAR_SITES_URL: &str = "https://opengeo.ncep.noaa.gov/geoserver/nws/ows?service=WFS&version=1.0.0&request=GetFeature&typeName=nws%3Aradar_sites&outputFormat=application%2Fjson";
pub const SNAPSHOT_MAX_AGE_DAYS: i64 = 31;

const SNAPSHOT_SCHEMA_VERSION: u32 = 1;
const REGISTRY_COORDINATE_TOLERANCE: f64 = 0.01;
const SOURCE_COORDINATE_TOLERANCE: f64 = 0.0001;
const SOURCE_ELEVATION_TOLERANCE_METERS: f64 = 1.0;
const EXCLUDED_SITE_IDS: &[&str] = &["KBIX", "KCRI", "KLIX", "KOUN"];

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RadarSiteSnapshot {
    pub schema_version: u32,
    pub last_verified: String,
    pub source: String,
    pub sites: Vec<RadarSite>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RadarSite {
    pub id: String,
    pub registry_name: String,
    pub source_name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation_meters: f64,
}

#[derive(Debug, Deserialize)]
struct FeatureCollection {
    features: Vec<Feature>,
}

#[derive(Debug, Deserialize)]
struct Feature {
    properties: FeatureProperties,
}

#[derive(Debug, Deserialize)]
struct FeatureProperties {
    rda_id: String,
    name: String,
    lat: f64,
    lon: f64,
    elevmeter: f64,
}

pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask must be in the workspace root")
        .to_path_buf()
}

pub fn snapshot_path() -> PathBuf {
    workspace_root().join("nexrad-model/data/operational-radar-sites.json")
}

pub fn candidate_path() -> PathBuf {
    workspace_root().join("target/radar-sites-current.json")
}

pub fn load_snapshot(path: &Path) -> Result<RadarSiteSnapshot> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("read radar site snapshot {}", path.display()))?;
    serde_json::from_str(&contents)
        .with_context(|| format!("parse radar site snapshot {}", path.display()))
}

pub fn snapshot_age_days(snapshot: &RadarSiteSnapshot, today: NaiveDate) -> Result<i64> {
    let verified = NaiveDate::parse_from_str(&snapshot.last_verified, "%Y-%m-%d")
        .context("parse snapshot last_verified as YYYY-MM-DD")?;
    let age = today.signed_duration_since(verified).num_days();
    if age < 0 {
        bail!(
            "radar site snapshot last_verified {} is in the future",
            snapshot.last_verified
        );
    }
    Ok(age)
}

pub fn is_snapshot_stale(snapshot: &RadarSiteSnapshot, today: NaiveDate) -> Result<bool> {
    Ok(snapshot_age_days(snapshot, today)? > SNAPSHOT_MAX_AGE_DAYS)
}

pub async fn fetch_operational_sites() -> Result<Vec<RadarSite>> {
    let client = reqwest::Client::builder()
        .user_agent("nexrad-xtask radar-site-registry-audit")
        .build()
        .context("build NOAA radar site client")?;
    let response = client
        .get(NOAA_RADAR_SITES_URL)
        .send()
        .await
        .context("download NOAA radar site GeoJSON")?
        .error_for_status()
        .context("NOAA radar site GeoJSON returned an error")?;
    let body = response
        .text()
        .await
        .context("read NOAA radar site GeoJSON")?;
    let collection: FeatureCollection =
        serde_json::from_str(&body).context("parse NOAA radar site GeoJSON")?;

    let registry_names: BTreeMap<_, _> = registry::sites()
        .iter()
        .map(|site| (site.id, site.city))
        .collect();
    let mut sites = BTreeMap::new();

    for feature in collection.features {
        let properties = feature.properties;
        let id = properties.rda_id.trim().to_ascii_uppercase();
        if !is_operational_us_wsr88d(&id) {
            continue;
        }

        let site = RadarSite {
            registry_name: registry_names
                .get(id.as_str())
                .copied()
                .unwrap_or(properties.name.as_str())
                .to_string(),
            source_name: properties.name.trim().to_string(),
            latitude: properties.lat,
            longitude: properties.lon,
            elevation_meters: properties.elevmeter,
            id: id.clone(),
        };

        if sites.insert(id.clone(), site).is_some() {
            bail!("NOAA radar site source contains duplicate identifier {id}");
        }
    }

    let sites: Vec<_> = sites.into_values().collect();
    if sites.is_empty() {
        bail!("NOAA radar site source contained no operational WSR-88D sites");
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

pub fn write_snapshot(snapshot: &RadarSiteSnapshot, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create snapshot directory {}", parent.display()))?;
    }
    let mut serialized = serde_json::to_string_pretty(snapshot).context("serialize snapshot")?;
    serialized.push('\n');
    fs::write(path, serialized).with_context(|| format!("write snapshot {}", path.display()))
}

pub async fn check_radar_sites(force_live: bool) -> Result<()> {
    let path = snapshot_path();
    let snapshot = load_snapshot(&path)?;
    let registry_differences = compare_registry(&snapshot);
    if !registry_differences.is_empty() {
        bail!(format_differences(
            "local registry differs from the checked-in radar site snapshot",
            &registry_differences
        ));
    }

    let today = Utc::now().date_naive();
    let age = snapshot_age_days(&snapshot, today)?;
    if !force_live && age <= SNAPSHOT_MAX_AGE_DAYS {
        return Ok(());
    }

    let live_sites = fetch_operational_sites().await?;
    let source_differences = compare_source(&snapshot, &live_sites);
    if source_differences.is_empty() {
        return Ok(());
    }

    let candidate = snapshot_from_live(live_sites, today);
    let candidate_path = candidate_path();
    write_snapshot(&candidate, &candidate_path)?;
    bail!(
        "{}\n\nCandidate snapshot written to {}.\nRun: cargo run -p xtask -- update-radar-sites",
        format_differences(
            "NOAA operational radar site data differs from the checked-in snapshot",
            &source_differences
        ),
        candidate_path.display()
    )
}

pub async fn update_radar_sites() -> Result<()> {
    let today = Utc::now().date_naive();
    let snapshot = snapshot_from_live(fetch_operational_sites().await?, today);
    let path = snapshot_path();
    write_snapshot(&snapshot, &path)?;
    println!("Updated {}", path.display());

    let differences = compare_registry(&snapshot);
    if !differences.is_empty() {
        bail!(format_differences(
            "snapshot updated, but the Rust registry still needs changes",
            &differences
        ));
    }
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
    fn snapshot_becomes_stale_after_31_days() {
        let snapshot = RadarSiteSnapshot {
            schema_version: 1,
            last_verified: "2026-01-01".to_string(),
            source: NOAA_RADAR_SITES_URL.to_string(),
            sites: Vec::new(),
        };

        assert!(
            !is_snapshot_stale(&snapshot, NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()).unwrap()
        );
        assert!(
            is_snapshot_stale(&snapshot, NaiveDate::from_ymd_opt(2026, 2, 2).unwrap()).unwrap()
        );
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
}
