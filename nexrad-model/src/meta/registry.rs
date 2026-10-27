//! Static registry of NEXRAD radar sites.
//!
//! This module provides a compile-time registry of all operational NEXRAD WSR-88D radar
//! sites in the United States and its territories. Operational status is sourced from the
//! [NOAA Radar Operations Center](https://www.roc.noaa.gov/build-loaded.php), with location data
//! from the [NCEI station report](https://www.ncei.noaa.gov/access/homr/file/nexrad-stations.txt).
//!
//! # Stability
//!
//! The list of sites is data, not API. Minor releases may add, correct or remove entries, and
//! may add sites of other radar kinds, such as Terminal Doppler Weather Radar (TDWR) sites, so
//! code should not assume that every entry is a WSR-88D or that the number of entries is fixed.
//! [`sites`] returns whatever the registry lists, and [`nearest_site`] returns the nearest
//! listed site. [`SiteEntry`] is `#[non_exhaustive]`, so fields can be added in a minor release
//! as well; read its fields rather than constructing or destructuring it exhaustively.
//!
//! The registry carries no feedhorn height. A scan's own [`Scan::site`](crate::data::Scan::site),
//! decoded from the volume data block, carries both the site elevation and the feedhorn height
//! and is the preferred source for a [`RadarCoordinateSystem`](crate::geo::RadarCoordinateSystem).

use crate::result::{Error, Result};

/// Metadata for a NEXRAD radar site in the static registry.
///
/// This struct is `#[non_exhaustive]`: minor releases may add fields, so it cannot be built
/// with a struct expression or matched without `..` outside this crate. Entries come from
/// [`sites`], [`site_by_id`] and [`nearest_site`]. See the [module documentation](self) for
/// how the set of entries may change.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct SiteEntry {
    /// Four-letter ICAO identifier (e.g., "KTLX").
    pub id: &'static str,
    /// City or location name (e.g., "Oklahoma City").
    pub city: &'static str,
    /// Two-letter state or territory abbreviation (e.g., `Some("OK")`), or `None` for a site
    /// outside the United States.
    pub state: Option<&'static str>,
    /// Latitude in degrees.
    pub latitude: f64,
    /// Longitude in degrees.
    pub longitude: f64,
    /// Site elevation above mean sea level in meters.
    pub site_elevation_meters: i16,
}

impl SiteEntry {
    /// Create a [`super::Site`] from this registry entry.
    ///
    /// The registry carries no feedhorn height, so the site's
    /// [`feedhorn_height_meters`](super::Site::feedhorn_height_meters) is `0` and a
    /// [`RadarCoordinateSystem`](crate::geo::RadarCoordinateSystem) built from it places the
    /// antenna at ground level. Prefer a scan's own
    /// [`Scan::site`](crate::data::Scan::site), decoded from the volume data block, as the
    /// source for coordinate systems.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidSite`] if the entry's identifier is not four ASCII alphanumeric
    /// characters or its location is out of range, which no entry in the registry is.
    pub fn to_site(&self) -> Result<super::Site> {
        let identifier: [u8; 4] =
            self.id
                .as_bytes()
                .try_into()
                .map_err(|_| Error::InvalidSite {
                    parameter: "identifier",
                    reason: "must be four ASCII alphanumeric characters",
                })?;

        super::Site::builder(identifier, self.latitude, self.longitude)
            .site_elevation_meters(self.site_elevation_meters)
            .build()
    }
}

impl std::fmt::Display for SiteEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.state {
            Some(state) => write!(f, "{} ({}, {})", self.id, self.city, state),
            None => write!(f, "{} ({})", self.id, self.city),
        }
    }
}

/// Returns every site in the registry, in no particular order.
///
/// Minor releases may add, correct or remove entries, including sites of other radar kinds
/// such as TDWR; see the [module documentation](self).
pub fn sites() -> &'static [SiteEntry] {
    SITES
}

/// Look up a NEXRAD radar site by its ICAO identifier (case-insensitive).
///
/// # Example
///
/// ```
/// use nexrad_model::meta::registry;
///
/// let site = registry::site_by_id("KTLX").unwrap();
/// assert_eq!(site.city, "Oklahoma City");
/// assert_eq!(site.state, Some("OK"));
/// ```
pub fn site_by_id(id: &str) -> Option<&'static SiteEntry> {
    let id_upper = id.to_uppercase();
    SITES.iter().find(|s| s.id == id_upper)
}

/// Find the listed site nearest to a given latitude/longitude.
///
/// Compares great-circle (haversine) distances on a spherical earth, so the result is correct
/// across the antimeridian and near the poles. The result is the nearest entry of [`sites`],
/// so it can change when a minor release adds, corrects or removes entries, including when
/// sites of another radar kind are added; see the [module documentation](self). Returns `None`
/// only if the registry is empty.
///
/// # Example
///
/// ```
/// use nexrad_model::meta::registry;
///
/// // Find the nearest radar to downtown Oklahoma City
/// let site = registry::nearest_site(35.4676, -97.5164).unwrap();
/// assert_eq!(site.id, "KTLX");
/// ```
pub fn nearest_site(latitude: f64, longitude: f64) -> Option<&'static SiteEntry> {
    SITES.iter().min_by(|a, b| {
        let dist_a = haversine_central_angle(latitude, longitude, a.latitude, a.longitude);
        let dist_b = haversine_central_angle(latitude, longitude, b.latitude, b.longitude);
        dist_a.total_cmp(&dist_b)
    })
}

/// The great-circle central angle in radians between two points given in degrees, by the
/// haversine formula. Longitude differences wrap across the antimeridian.
fn haversine_central_angle(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let (lat1, lat2) = (lat1.to_radians(), lat2.to_radians());
    let d_lat = lat2 - lat1;
    let d_lon = (lon2 - lon1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
    2.0 * a.sqrt().min(1.0).asin()
}

/// Static registry of all NEXRAD WSR-88D radar sites.
///
/// Data sourced from NOAA Radar Operations Center and NWS documentation.
/// Coordinates and elevations are approximate.
static SITES: &[SiteEntry] = &[
    // Alabama
    SiteEntry {
        id: "KBMX",
        city: "Birmingham",
        state: Some("AL"),
        latitude: 33.1722,
        longitude: -86.7698,
        site_elevation_meters: 197,
    },
    SiteEntry {
        id: "KEOX",
        city: "Fort Rucker",
        state: Some("AL"),
        latitude: 31.4606,
        longitude: -85.4592,
        site_elevation_meters: 132,
    },
    SiteEntry {
        id: "KHTX",
        city: "Huntsville",
        state: Some("AL"),
        latitude: 34.9306,
        longitude: -86.0833,
        site_elevation_meters: 536,
    },
    SiteEntry {
        id: "KMOB",
        city: "Mobile",
        state: Some("AL"),
        latitude: 30.6794,
        longitude: -88.2397,
        site_elevation_meters: 63,
    },
    SiteEntry {
        id: "KMXX",
        city: "Maxwell AFB",
        state: Some("AL"),
        latitude: 32.5367,
        longitude: -85.7897,
        site_elevation_meters: 122,
    },
    // Alaska
    SiteEntry {
        id: "PABC",
        city: "Bethel",
        state: Some("AK"),
        latitude: 60.7919,
        longitude: -161.8764,
        site_elevation_meters: 48,
    },
    SiteEntry {
        id: "PACG",
        city: "Sitka",
        state: Some("AK"),
        latitude: 56.8525,
        longitude: -135.5294,
        site_elevation_meters: 63,
    },
    SiteEntry {
        id: "PAEC",
        city: "Nome",
        state: Some("AK"),
        latitude: 64.5114,
        longitude: -165.295,
        site_elevation_meters: 16,
    },
    SiteEntry {
        id: "PAHG",
        city: "Anchorage",
        state: Some("AK"),
        latitude: 60.7258,
        longitude: -151.3514,
        site_elevation_meters: 74,
    },
    SiteEntry {
        id: "PAIH",
        city: "Middleton Island",
        state: Some("AK"),
        latitude: 59.4614,
        longitude: -146.3031,
        site_elevation_meters: 20,
    },
    SiteEntry {
        id: "PAKC",
        city: "King Salmon",
        state: Some("AK"),
        latitude: 58.6794,
        longitude: -156.6297,
        site_elevation_meters: 19,
    },
    SiteEntry {
        id: "PAPD",
        city: "Fairbanks",
        state: Some("AK"),
        latitude: 65.0351,
        longitude: -147.5014,
        site_elevation_meters: 825,
    },
    // Arizona
    SiteEntry {
        id: "KEMX",
        city: "Tucson",
        state: Some("AZ"),
        latitude: 31.8939,
        longitude: -110.6303,
        site_elevation_meters: 1586,
    },
    SiteEntry {
        id: "KFSX",
        city: "Flagstaff",
        state: Some("AZ"),
        latitude: 34.5744,
        longitude: -111.1983,
        site_elevation_meters: 2261,
    },
    SiteEntry {
        id: "KIWA",
        city: "Phoenix",
        state: Some("AZ"),
        latitude: 33.2892,
        longitude: -111.67,
        site_elevation_meters: 413,
    },
    SiteEntry {
        id: "KYUX",
        city: "Yuma",
        state: Some("AZ"),
        latitude: 32.4953,
        longitude: -114.6567,
        site_elevation_meters: 53,
    },
    // Arkansas
    SiteEntry {
        id: "KLZK",
        city: "Little Rock",
        state: Some("AR"),
        latitude: 34.8364,
        longitude: -92.2622,
        site_elevation_meters: 173,
    },
    SiteEntry {
        id: "KSRX",
        city: "Fort Smith",
        state: Some("AR"),
        latitude: 35.2906,
        longitude: -94.3619,
        site_elevation_meters: 195,
    },
    // California
    SiteEntry {
        id: "KBBX",
        city: "Beale AFB",
        state: Some("CA"),
        latitude: 39.4961,
        longitude: -121.6317,
        site_elevation_meters: 53,
    },
    SiteEntry {
        id: "KBHX",
        city: "Eureka",
        state: Some("CA"),
        latitude: 40.4986,
        longitude: -124.2922,
        site_elevation_meters: 767,
    },
    SiteEntry {
        id: "KEYX",
        city: "Edwards AFB",
        state: Some("CA"),
        latitude: 35.0978,
        longitude: -117.5608,
        site_elevation_meters: 840,
    },
    SiteEntry {
        id: "KHNX",
        city: "Hanford",
        state: Some("CA"),
        latitude: 36.3142,
        longitude: -119.6322,
        site_elevation_meters: 74,
    },
    SiteEntry {
        id: "KMUX",
        city: "San Francisco",
        state: Some("CA"),
        latitude: 37.1553,
        longitude: -121.8983,
        site_elevation_meters: 1057,
    },
    SiteEntry {
        id: "KNKX",
        city: "San Diego",
        state: Some("CA"),
        latitude: 32.9189,
        longitude: -117.0419,
        site_elevation_meters: 291,
    },
    SiteEntry {
        id: "KSOX",
        city: "Santa Ana Mountains",
        state: Some("CA"),
        latitude: 33.8178,
        longitude: -117.6358,
        site_elevation_meters: 923,
    },
    SiteEntry {
        id: "KVBX",
        city: "Vandenberg AFB",
        state: Some("CA"),
        latitude: 34.8383,
        longitude: -120.3978,
        site_elevation_meters: 373,
    },
    SiteEntry {
        id: "KVTX",
        city: "Los Angeles",
        state: Some("CA"),
        latitude: 34.4114,
        longitude: -119.1794,
        site_elevation_meters: 831,
    },
    SiteEntry {
        id: "KDAX",
        city: "Sacramento",
        state: Some("CA"),
        latitude: 38.5011,
        longitude: -121.6778,
        site_elevation_meters: 9,
    },
    // Colorado
    SiteEntry {
        id: "KFTG",
        city: "Denver",
        state: Some("CO"),
        latitude: 39.7867,
        longitude: -104.5458,
        site_elevation_meters: 1675,
    },
    SiteEntry {
        id: "KGJX",
        city: "Grand Junction",
        state: Some("CO"),
        latitude: 39.0622,
        longitude: -108.2139,
        site_elevation_meters: 3045,
    },
    SiteEntry {
        id: "KPUX",
        city: "Pueblo",
        state: Some("CO"),
        latitude: 38.4594,
        longitude: -104.1817,
        site_elevation_meters: 1600,
    },
    // Delaware
    SiteEntry {
        id: "KDOX",
        city: "Dover AFB",
        state: Some("DE"),
        latitude: 38.8258,
        longitude: -75.4401,
        site_elevation_meters: 50,
    },
    // Connecticut / New England
    SiteEntry {
        id: "KOKX",
        city: "Upton",
        state: Some("NY"),
        latitude: 40.8656,
        longitude: -72.8639,
        site_elevation_meters: 26,
    },
    // Florida
    SiteEntry {
        id: "KAMX",
        city: "Miami",
        state: Some("FL"),
        latitude: 25.6111,
        longitude: -80.4128,
        site_elevation_meters: 4,
    },
    SiteEntry {
        id: "KEVX",
        city: "Eglin AFB",
        state: Some("FL"),
        latitude: 30.5644,
        longitude: -85.9214,
        site_elevation_meters: 43,
    },
    SiteEntry {
        id: "KJAX",
        city: "Jacksonville",
        state: Some("FL"),
        latitude: 30.4847,
        longitude: -81.7019,
        site_elevation_meters: 10,
    },
    SiteEntry {
        id: "KMLB",
        city: "Melbourne",
        state: Some("FL"),
        latitude: 28.1133,
        longitude: -80.6542,
        site_elevation_meters: 11,
    },
    SiteEntry {
        id: "KTBW",
        city: "Tampa Bay",
        state: Some("FL"),
        latitude: 27.7056,
        longitude: -82.4017,
        site_elevation_meters: 13,
    },
    SiteEntry {
        id: "KTLH",
        city: "Tallahassee",
        state: Some("FL"),
        latitude: 30.3975,
        longitude: -84.3289,
        site_elevation_meters: 19,
    },
    SiteEntry {
        id: "KBYX",
        city: "Key West",
        state: Some("FL"),
        latitude: 24.5975,
        longitude: -81.7031,
        site_elevation_meters: 3,
    },
    // Georgia
    SiteEntry {
        id: "KFFC",
        city: "Atlanta",
        state: Some("GA"),
        latitude: 33.3636,
        longitude: -84.5658,
        site_elevation_meters: 262,
    },
    SiteEntry {
        id: "KJGX",
        city: "Robins AFB",
        state: Some("GA"),
        latitude: 32.6756,
        longitude: -83.3511,
        site_elevation_meters: 159,
    },
    SiteEntry {
        id: "KVAX",
        city: "Moody AFB",
        state: Some("GA"),
        latitude: 30.8900,
        longitude: -83.0019,
        site_elevation_meters: 54,
    },
    // Hawaii
    SiteEntry {
        id: "PHKI",
        city: "South Kauai",
        state: Some("HI"),
        latitude: 21.8939,
        longitude: -159.5522,
        site_elevation_meters: 55,
    },
    SiteEntry {
        id: "PHKM",
        city: "Kamuela",
        state: Some("HI"),
        latitude: 20.1253,
        longitude: -155.7781,
        site_elevation_meters: 1161,
    },
    SiteEntry {
        id: "PHMO",
        city: "Molokai",
        state: Some("HI"),
        latitude: 21.1328,
        longitude: -157.1803,
        site_elevation_meters: 416,
    },
    SiteEntry {
        id: "PHWA",
        city: "South Shore",
        state: Some("HI"),
        latitude: 19.0950,
        longitude: -155.5689,
        site_elevation_meters: 421,
    },
    // Idaho
    SiteEntry {
        id: "KCBX",
        city: "Boise",
        state: Some("ID"),
        latitude: 43.4908,
        longitude: -116.2358,
        site_elevation_meters: 933,
    },
    SiteEntry {
        id: "KSFX",
        city: "Pocatello",
        state: Some("ID"),
        latitude: 43.1058,
        longitude: -112.6861,
        site_elevation_meters: 1364,
    },
    // Illinois
    SiteEntry {
        id: "KILX",
        city: "Lincoln",
        state: Some("IL"),
        latitude: 40.1506,
        longitude: -89.3369,
        site_elevation_meters: 177,
    },
    SiteEntry {
        id: "KLOT",
        city: "Chicago",
        state: Some("IL"),
        latitude: 41.6044,
        longitude: -88.0847,
        site_elevation_meters: 202,
    },
    // Indiana
    SiteEntry {
        id: "KIND",
        city: "Indianapolis",
        state: Some("IN"),
        latitude: 39.7075,
        longitude: -86.2803,
        site_elevation_meters: 241,
    },
    SiteEntry {
        id: "KIWX",
        city: "Fort Wayne",
        state: Some("IN"),
        latitude: 41.3586,
        longitude: -85.7000,
        site_elevation_meters: 293,
    },
    SiteEntry {
        id: "KVWX",
        city: "Evansville",
        state: Some("IN"),
        latitude: 38.2603,
        longitude: -87.7247,
        site_elevation_meters: 155,
    },
    // Iowa
    SiteEntry {
        id: "KDMX",
        city: "Des Moines",
        state: Some("IA"),
        latitude: 41.7311,
        longitude: -93.7228,
        site_elevation_meters: 299,
    },
    SiteEntry {
        id: "KDVN",
        city: "Davenport",
        state: Some("IA"),
        latitude: 41.6117,
        longitude: -90.5808,
        site_elevation_meters: 230,
    },
    // Kansas
    SiteEntry {
        id: "KDDC",
        city: "Dodge City",
        state: Some("KS"),
        latitude: 37.7608,
        longitude: -99.9689,
        site_elevation_meters: 790,
    },
    SiteEntry {
        id: "KGLD",
        city: "Goodland",
        state: Some("KS"),
        latitude: 39.3669,
        longitude: -101.7003,
        site_elevation_meters: 1113,
    },
    SiteEntry {
        id: "KICT",
        city: "Wichita",
        state: Some("KS"),
        latitude: 37.6544,
        longitude: -97.4428,
        site_elevation_meters: 407,
    },
    SiteEntry {
        id: "KTWX",
        city: "Topeka",
        state: Some("KS"),
        latitude: 38.9969,
        longitude: -96.2325,
        site_elevation_meters: 417,
    },
    // Kentucky
    SiteEntry {
        id: "KHPX",
        city: "Fort Campbell",
        state: Some("KY"),
        latitude: 36.7369,
        longitude: -87.2850,
        site_elevation_meters: 177,
    },
    SiteEntry {
        id: "KJKL",
        city: "Jackson",
        state: Some("KY"),
        latitude: 37.5908,
        longitude: -83.3131,
        site_elevation_meters: 415,
    },
    SiteEntry {
        id: "KLVX",
        city: "Louisville",
        state: Some("KY"),
        latitude: 37.9753,
        longitude: -85.9439,
        site_elevation_meters: 219,
    },
    SiteEntry {
        id: "KPAH",
        city: "Paducah",
        state: Some("KY"),
        latitude: 37.0683,
        longitude: -88.7722,
        site_elevation_meters: 119,
    },
    // Louisiana
    SiteEntry {
        id: "KHGX",
        city: "Houston",
        state: Some("TX"),
        latitude: 29.4719,
        longitude: -95.0792,
        site_elevation_meters: 5,
    },
    SiteEntry {
        id: "KLCH",
        city: "Lake Charles",
        state: Some("LA"),
        latitude: 30.1253,
        longitude: -93.2158,
        site_elevation_meters: 4,
    },
    SiteEntry {
        id: "KHDC",
        city: "Hammond",
        state: Some("LA"),
        latitude: 30.5193,
        longitude: -90.4074,
        site_elevation_meters: 13,
    },
    SiteEntry {
        id: "KPOE",
        city: "Fort Polk",
        state: Some("LA"),
        latitude: 31.1556,
        longitude: -92.9756,
        site_elevation_meters: 124,
    },
    SiteEntry {
        id: "KSHV",
        city: "Shreveport",
        state: Some("LA"),
        latitude: 32.4508,
        longitude: -93.8411,
        site_elevation_meters: 83,
    },
    // Maine
    SiteEntry {
        id: "KCBW",
        city: "Caribou",
        state: Some("ME"),
        latitude: 46.0392,
        longitude: -67.8067,
        site_elevation_meters: 227,
    },
    SiteEntry {
        id: "KGYX",
        city: "Portland",
        state: Some("ME"),
        latitude: 43.8914,
        longitude: -70.2567,
        site_elevation_meters: 125,
    },
    // Maryland
    SiteEntry {
        id: "KLWX",
        city: "Sterling",
        state: Some("VA"),
        latitude: 38.9753,
        longitude: -77.4778,
        site_elevation_meters: 83,
    },
    // Massachusetts
    SiteEntry {
        id: "KBOX",
        city: "Boston",
        state: Some("MA"),
        latitude: 41.9558,
        longitude: -71.1369,
        site_elevation_meters: 36,
    },
    // Michigan
    SiteEntry {
        id: "KAPX",
        city: "Gaylord",
        state: Some("MI"),
        latitude: 44.9072,
        longitude: -84.7197,
        site_elevation_meters: 446,
    },
    SiteEntry {
        id: "KDTX",
        city: "Detroit",
        state: Some("MI"),
        latitude: 42.6997,
        longitude: -83.4717,
        site_elevation_meters: 327,
    },
    SiteEntry {
        id: "KGRR",
        city: "Grand Rapids",
        state: Some("MI"),
        latitude: 42.8939,
        longitude: -85.5447,
        site_elevation_meters: 237,
    },
    SiteEntry {
        id: "KMQT",
        city: "Marquette",
        state: Some("MI"),
        latitude: 46.5314,
        longitude: -87.5486,
        site_elevation_meters: 430,
    },
    // Minnesota
    SiteEntry {
        id: "KDLH",
        city: "Duluth",
        state: Some("MN"),
        latitude: 46.8369,
        longitude: -92.2097,
        site_elevation_meters: 435,
    },
    SiteEntry {
        id: "KMPX",
        city: "Minneapolis",
        state: Some("MN"),
        latitude: 44.8489,
        longitude: -93.5653,
        site_elevation_meters: 288,
    },
    // Mississippi
    SiteEntry {
        id: "KDGX",
        city: "Brandon",
        state: Some("MS"),
        latitude: 32.2800,
        longitude: -89.9844,
        site_elevation_meters: 149,
    },
    SiteEntry {
        id: "KGWX",
        city: "Columbus AFB",
        state: Some("MS"),
        latitude: 33.8967,
        longitude: -88.3289,
        site_elevation_meters: 145,
    },
    // Missouri
    SiteEntry {
        id: "KEAX",
        city: "Kansas City",
        state: Some("MO"),
        latitude: 38.8103,
        longitude: -94.2644,
        site_elevation_meters: 303,
    },
    SiteEntry {
        id: "KLSX",
        city: "St. Louis",
        state: Some("MO"),
        latitude: 38.6986,
        longitude: -90.6828,
        site_elevation_meters: 185,
    },
    SiteEntry {
        id: "KSGF",
        city: "Springfield",
        state: Some("MO"),
        latitude: 37.2353,
        longitude: -93.4006,
        site_elevation_meters: 390,
    },
    // Montana
    SiteEntry {
        id: "KBLX",
        city: "Billings",
        state: Some("MT"),
        latitude: 45.8536,
        longitude: -108.6069,
        site_elevation_meters: 1097,
    },
    SiteEntry {
        id: "KGGW",
        city: "Glasgow",
        state: Some("MT"),
        latitude: 48.2064,
        longitude: -106.6253,
        site_elevation_meters: 694,
    },
    SiteEntry {
        id: "KMSX",
        city: "Missoula",
        state: Some("MT"),
        latitude: 47.0411,
        longitude: -113.9864,
        site_elevation_meters: 2394,
    },
    SiteEntry {
        id: "KTFX",
        city: "Great Falls",
        state: Some("MT"),
        latitude: 47.4597,
        longitude: -111.3853,
        site_elevation_meters: 1132,
    },
    // Nebraska
    SiteEntry {
        id: "KLNX",
        city: "North Platte",
        state: Some("NE"),
        latitude: 41.9578,
        longitude: -100.5761,
        site_elevation_meters: 906,
    },
    SiteEntry {
        id: "KOAX",
        city: "Omaha",
        state: Some("NE"),
        latitude: 41.3203,
        longitude: -96.3667,
        site_elevation_meters: 350,
    },
    SiteEntry {
        id: "KUEX",
        city: "Hastings",
        state: Some("NE"),
        latitude: 40.3208,
        longitude: -98.4419,
        site_elevation_meters: 602,
    },
    // Nevada
    SiteEntry {
        id: "KLRX",
        city: "Elko",
        state: Some("NV"),
        latitude: 40.7400,
        longitude: -116.8025,
        site_elevation_meters: 2056,
    },
    SiteEntry {
        id: "KESX",
        city: "Las Vegas",
        state: Some("NV"),
        latitude: 35.7011,
        longitude: -114.8914,
        site_elevation_meters: 1483,
    },
    SiteEntry {
        id: "KRGX",
        city: "Reno",
        state: Some("NV"),
        latitude: 39.7542,
        longitude: -119.4622,
        site_elevation_meters: 2530,
    },
    // New Mexico
    SiteEntry {
        id: "KABX",
        city: "Albuquerque",
        state: Some("NM"),
        latitude: 35.1497,
        longitude: -106.8239,
        site_elevation_meters: 1789,
    },
    SiteEntry {
        id: "KFDX",
        city: "Cannon AFB",
        state: Some("NM"),
        latitude: 34.6350,
        longitude: -103.6189,
        site_elevation_meters: 1417,
    },
    SiteEntry {
        id: "KHDX",
        city: "Holloman AFB",
        state: Some("NM"),
        latitude: 33.0769,
        longitude: -106.12,
        site_elevation_meters: 1287,
    },
    // New York
    SiteEntry {
        id: "KBGM",
        city: "Binghamton",
        state: Some("NY"),
        latitude: 42.1997,
        longitude: -75.9847,
        site_elevation_meters: 490,
    },
    SiteEntry {
        id: "KBUF",
        city: "Buffalo",
        state: Some("NY"),
        latitude: 42.9486,
        longitude: -78.7369,
        site_elevation_meters: 211,
    },
    SiteEntry {
        id: "KENX",
        city: "Albany",
        state: Some("NY"),
        latitude: 42.5864,
        longitude: -74.0639,
        site_elevation_meters: 557,
    },
    SiteEntry {
        id: "KTYX",
        city: "Montague",
        state: Some("NY"),
        latitude: 43.7558,
        longitude: -75.6800,
        site_elevation_meters: 562,
    },
    // North Carolina
    SiteEntry {
        id: "KMHX",
        city: "Morehead City",
        state: Some("NC"),
        latitude: 34.7761,
        longitude: -76.8764,
        site_elevation_meters: 9,
    },
    SiteEntry {
        id: "KRAX",
        city: "Raleigh",
        state: Some("NC"),
        latitude: 35.6656,
        longitude: -78.4903,
        site_elevation_meters: 106,
    },
    SiteEntry {
        id: "KLTX",
        city: "Wilmington",
        state: Some("NC"),
        latitude: 33.9892,
        longitude: -78.4292,
        site_elevation_meters: 19,
    },
    // North Dakota
    SiteEntry {
        id: "KBIS",
        city: "Bismarck",
        state: Some("ND"),
        latitude: 46.7708,
        longitude: -100.7603,
        site_elevation_meters: 505,
    },
    SiteEntry {
        id: "KMVX",
        city: "Fargo",
        state: Some("ND"),
        latitude: 47.5281,
        longitude: -97.3256,
        site_elevation_meters: 301,
    },
    SiteEntry {
        id: "KMBX",
        city: "Minot AFB",
        state: Some("ND"),
        latitude: 48.3925,
        longitude: -100.8644,
        site_elevation_meters: 455,
    },
    // Ohio
    SiteEntry {
        id: "KCLE",
        city: "Cleveland",
        state: Some("OH"),
        latitude: 41.4131,
        longitude: -81.8597,
        site_elevation_meters: 233,
    },
    SiteEntry {
        id: "KILN",
        city: "Wilmington",
        state: Some("OH"),
        latitude: 39.4203,
        longitude: -83.8217,
        site_elevation_meters: 322,
    },
    // Oklahoma
    SiteEntry {
        id: "KFDR",
        city: "Frederick",
        state: Some("OK"),
        latitude: 34.3622,
        longitude: -98.9764,
        site_elevation_meters: 386,
    },
    SiteEntry {
        id: "KINX",
        city: "Tulsa",
        state: Some("OK"),
        latitude: 36.1750,
        longitude: -95.5647,
        site_elevation_meters: 204,
    },
    SiteEntry {
        id: "KTLX",
        city: "Oklahoma City",
        state: Some("OK"),
        latitude: 35.3331,
        longitude: -97.2778,
        site_elevation_meters: 370,
    },
    SiteEntry {
        id: "KVNX",
        city: "Vance AFB",
        state: Some("OK"),
        latitude: 36.7406,
        longitude: -98.1278,
        site_elevation_meters: 369,
    },
    // Oregon
    SiteEntry {
        id: "KMAX",
        city: "Medford",
        state: Some("OR"),
        latitude: 42.0811,
        longitude: -122.7167,
        site_elevation_meters: 2290,
    },
    SiteEntry {
        id: "KPDT",
        city: "Pendleton",
        state: Some("OR"),
        latitude: 45.6906,
        longitude: -118.8528,
        site_elevation_meters: 462,
    },
    SiteEntry {
        id: "KRTX",
        city: "Portland",
        state: Some("OR"),
        latitude: 45.7150,
        longitude: -122.9653,
        site_elevation_meters: 479,
    },
    // Pennsylvania
    SiteEntry {
        id: "KCCX",
        city: "State College",
        state: Some("PA"),
        latitude: 40.9231,
        longitude: -78.0039,
        site_elevation_meters: 733,
    },
    SiteEntry {
        id: "KDIX",
        city: "Philadelphia",
        state: Some("PA"),
        latitude: 39.9469,
        longitude: -74.4108,
        site_elevation_meters: 45,
    },
    SiteEntry {
        id: "KPBZ",
        city: "Pittsburgh",
        state: Some("PA"),
        latitude: 40.5317,
        longitude: -80.2179,
        site_elevation_meters: 361,
    },
    // Puerto Rico
    SiteEntry {
        id: "TJUA",
        city: "San Juan",
        state: Some("PR"),
        latitude: 18.1156,
        longitude: -66.0781,
        site_elevation_meters: 867,
    },
    // South Carolina
    SiteEntry {
        id: "KCAE",
        city: "Columbia",
        state: Some("SC"),
        latitude: 33.9486,
        longitude: -81.1186,
        site_elevation_meters: 70,
    },
    SiteEntry {
        id: "KCLX",
        city: "Charleston",
        state: Some("SC"),
        latitude: 32.6556,
        longitude: -81.0425,
        site_elevation_meters: 30,
    },
    SiteEntry {
        id: "KGSP",
        city: "Greenville",
        state: Some("SC"),
        latitude: 34.8833,
        longitude: -82.2200,
        site_elevation_meters: 296,
    },
    // South Dakota
    SiteEntry {
        id: "KABR",
        city: "Aberdeen",
        state: Some("SD"),
        latitude: 45.4558,
        longitude: -98.4131,
        site_elevation_meters: 397,
    },
    SiteEntry {
        id: "KUDX",
        city: "Rapid City",
        state: Some("SD"),
        latitude: 44.1250,
        longitude: -102.8297,
        site_elevation_meters: 919,
    },
    SiteEntry {
        id: "KFSD",
        city: "Sioux Falls",
        state: Some("SD"),
        latitude: 43.5878,
        longitude: -96.7289,
        site_elevation_meters: 436,
    },
    // Tennessee
    SiteEntry {
        id: "KMRX",
        city: "Knoxville",
        state: Some("TN"),
        latitude: 36.1686,
        longitude: -83.4017,
        site_elevation_meters: 408,
    },
    SiteEntry {
        id: "KNQA",
        city: "Memphis",
        state: Some("TN"),
        latitude: 35.3447,
        longitude: -89.8733,
        site_elevation_meters: 86,
    },
    SiteEntry {
        id: "KOHX",
        city: "Nashville",
        state: Some("TN"),
        latitude: 36.2472,
        longitude: -86.5625,
        site_elevation_meters: 176,
    },
    // Texas
    SiteEntry {
        id: "KAMA",
        city: "Amarillo",
        state: Some("TX"),
        latitude: 35.2333,
        longitude: -101.7092,
        site_elevation_meters: 1093,
    },
    SiteEntry {
        id: "KBRO",
        city: "Brownsville",
        state: Some("TX"),
        latitude: 25.9158,
        longitude: -97.4189,
        site_elevation_meters: 7,
    },
    SiteEntry {
        id: "KCRP",
        city: "Corpus Christi",
        state: Some("TX"),
        latitude: 27.7842,
        longitude: -97.5111,
        site_elevation_meters: 14,
    },
    SiteEntry {
        id: "KDFX",
        city: "Laughlin AFB",
        state: Some("TX"),
        latitude: 29.2722,
        longitude: -100.2803,
        site_elevation_meters: 345,
    },
    SiteEntry {
        id: "KDYX",
        city: "Dyess AFB",
        state: Some("TX"),
        latitude: 32.5386,
        longitude: -99.2544,
        site_elevation_meters: 463,
    },
    SiteEntry {
        id: "KEPZ",
        city: "El Paso",
        state: Some("TX"),
        latitude: 31.8731,
        longitude: -106.6978,
        site_elevation_meters: 1251,
    },
    SiteEntry {
        id: "KEWX",
        city: "San Antonio",
        state: Some("TX"),
        latitude: 29.7039,
        longitude: -98.0286,
        site_elevation_meters: 193,
    },
    SiteEntry {
        id: "KFWS",
        city: "Dallas/Fort Worth",
        state: Some("TX"),
        latitude: 32.5731,
        longitude: -97.3031,
        site_elevation_meters: 208,
    },
    SiteEntry {
        id: "KGRK",
        city: "Fort Hood",
        state: Some("TX"),
        latitude: 30.7217,
        longitude: -97.3828,
        site_elevation_meters: 164,
    },
    SiteEntry {
        id: "KLBB",
        city: "Lubbock",
        state: Some("TX"),
        latitude: 33.6542,
        longitude: -101.8142,
        site_elevation_meters: 993,
    },
    SiteEntry {
        id: "KMAF",
        city: "Midland/Odessa",
        state: Some("TX"),
        latitude: 31.9433,
        longitude: -102.1892,
        site_elevation_meters: 874,
    },
    SiteEntry {
        id: "KSJT",
        city: "San Angelo",
        state: Some("TX"),
        latitude: 31.3714,
        longitude: -100.4925,
        site_elevation_meters: 576,
    },
    // Utah
    SiteEntry {
        id: "KICX",
        city: "Cedar City",
        state: Some("UT"),
        latitude: 37.5911,
        longitude: -112.8622,
        site_elevation_meters: 3231,
    },
    SiteEntry {
        id: "KMTX",
        city: "Salt Lake City",
        state: Some("UT"),
        latitude: 41.2628,
        longitude: -112.4478,
        site_elevation_meters: 1969,
    },
    // Vermont
    SiteEntry {
        id: "KCXX",
        city: "Burlington",
        state: Some("VT"),
        latitude: 44.5111,
        longitude: -73.1667,
        site_elevation_meters: 97,
    },
    // Virginia
    SiteEntry {
        id: "KAKQ",
        city: "Wakefield",
        state: Some("VA"),
        latitude: 36.9839,
        longitude: -77.0072,
        site_elevation_meters: 34,
    },
    SiteEntry {
        id: "KFCX",
        city: "Roanoke",
        state: Some("VA"),
        latitude: 37.0242,
        longitude: -80.2744,
        site_elevation_meters: 874,
    },
    // Washington
    SiteEntry {
        id: "KATX",
        city: "Seattle",
        state: Some("WA"),
        latitude: 48.1944,
        longitude: -122.4958,
        site_elevation_meters: 151,
    },
    SiteEntry {
        id: "KLGX",
        city: "Langley Hill",
        state: Some("WA"),
        latitude: 47.1169,
        longitude: -124.1067,
        site_elevation_meters: 112,
    },
    SiteEntry {
        id: "KOTX",
        city: "Spokane",
        state: Some("WA"),
        latitude: 47.6806,
        longitude: -117.6267,
        site_elevation_meters: 728,
    },
    // West Virginia
    SiteEntry {
        id: "KRLX",
        city: "Charleston",
        state: Some("WV"),
        latitude: 38.3111,
        longitude: -81.7231,
        site_elevation_meters: 329,
    },
    // Wisconsin
    SiteEntry {
        id: "KARX",
        city: "La Crosse",
        state: Some("WI"),
        latitude: 43.8228,
        longitude: -91.1911,
        site_elevation_meters: 389,
    },
    SiteEntry {
        id: "KGRB",
        city: "Green Bay",
        state: Some("WI"),
        latitude: 44.4986,
        longitude: -88.1111,
        site_elevation_meters: 208,
    },
    SiteEntry {
        id: "KMKX",
        city: "Milwaukee",
        state: Some("WI"),
        latitude: 42.9678,
        longitude: -88.5506,
        site_elevation_meters: 292,
    },
    // Wyoming
    SiteEntry {
        id: "KCYS",
        city: "Cheyenne",
        state: Some("WY"),
        latitude: 41.1519,
        longitude: -104.8061,
        site_elevation_meters: 1868,
    },
    SiteEntry {
        id: "KRIW",
        city: "Riverton",
        state: Some("WY"),
        latitude: 43.0661,
        longitude: -108.4772,
        site_elevation_meters: 1697,
    },
    // Guam
    SiteEntry {
        id: "PGUA",
        city: "Andersen AFB",
        state: Some("GU"),
        latitude: 13.4544,
        longitude: 144.8111,
        site_elevation_meters: 78,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn registry_contains_unique_operational_us_sites() {
        let ids: HashSet<_> = sites().iter().map(|site| site.id).collect();

        // NOAA operates 159 sites, including three overseas DoD sites outside this registry's
        // documented United States and territories scope.
        assert_eq!(sites().len(), 156);
        assert_eq!(ids.len(), sites().len(), "site identifiers must be unique");

        for id in ["KBHX", "KDOX", "KHDC", "KLGX", "PAPD"] {
            assert!(site_by_id(id).is_some(), "missing operational site {id}");
        }

        for id in ["KABC", "KACG", "KAIH", "KAKC", "KLIX"] {
            assert!(site_by_id(id).is_none(), "unexpected obsolete site {id}");
        }
    }

    #[test]
    fn every_registry_entry_converts_to_a_valid_site() {
        for entry in sites() {
            let site = entry
                .to_site()
                .unwrap_or_else(|error| panic!("{} is invalid: {error}", entry.id));
            assert_eq!(site.identifier(), entry.id);
        }
    }

    #[test]
    fn nearest_site_finds_repaired_registry_entries() {
        for (latitude, longitude, expected) in [
            (40.4986, -124.2922, "KBHX"),
            (38.8258, -75.4401, "KDOX"),
            (30.5193, -90.4074, "KHDC"),
            (47.1169, -124.1067, "KLGX"),
            (65.0351, -147.5014, "PAPD"),
        ] {
            assert_eq!(
                nearest_site(latitude, longitude).map(|site| site.id),
                Some(expected)
            );
        }
    }
}
