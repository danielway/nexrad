//! Tests for the radar-physics context carried by the model: per-radial Nyquist velocity and
//! unambiguous range, the link between sweeps and their VCP elevation cuts, the VCP number set,
//! and the self-consistent geolocation model.

use nexrad_model::data::{
    ChannelConfiguration, ElevationCut, Radial, Scan, Sweep, VcpNumber, VolumeCoveragePattern,
    WaveformType,
};
use nexrad_model::geo::{GeoPoint, PolarPoint, RadarCoordinateSystem};
use nexrad_model::meta::{registry, Site};
use nexrad_model::result::Error;

const KNOWN_VCPS: [(u16, VcpNumber); 12] = [
    (11, VcpNumber::VCP_11),
    (12, VcpNumber::VCP_12),
    (21, VcpNumber::VCP_21),
    (31, VcpNumber::VCP_31),
    (32, VcpNumber::VCP_32),
    (35, VcpNumber::VCP_35),
    (112, VcpNumber::VCP_112),
    (121, VcpNumber::VCP_121),
    (211, VcpNumber::VCP_211),
    (212, VcpNumber::VCP_212),
    (215, VcpNumber::VCP_215),
    (221, VcpNumber::VCP_221),
];

#[test]
fn radial_round_trips_nyquist_velocity_and_unambiguous_range() {
    let radial = Radial::builder(1, 10.0, 1, 0.5)
        .nyquist_velocity_mps(26.4)
        .unambiguous_range_km(117.0)
        .build()
        .unwrap();
    assert_eq!(radial.nyquist_velocity_mps(), Some(26.4));
    assert_eq!(radial.unambiguous_range_km(), Some(117.0));

    let bare = Radial::builder(1, 10.0, 1, 0.5).build().unwrap();
    assert_eq!(bare.nyquist_velocity_mps(), None);
    assert_eq!(bare.unambiguous_range_km(), None);
}

#[cfg(feature = "uom")]
#[test]
fn radial_reports_nyquist_velocity_and_unambiguous_range_as_quantities() {
    use uom::si::{length::kilometer, velocity::meter_per_second};

    let radial = Radial::builder(1, 10.0, 1, 0.5)
        .nyquist_velocity_mps(26.5)
        .unambiguous_range_km(117.0)
        .build()
        .unwrap();
    let nyquist = radial.nyquist_velocity().unwrap();
    assert!((nyquist.get::<meter_per_second>() - 26.5).abs() < 1e-9);
    let range = radial.unambiguous_range().unwrap();
    assert!((range.get::<kilometer>() - 117.0).abs() < 1e-9);
}

#[test]
fn radial_rejects_invalid_nyquist_velocity_and_unambiguous_range() {
    for value in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        let result = Radial::builder(1, 10.0, 1, 0.5)
            .nyquist_velocity_mps(value)
            .build();
        assert!(
            matches!(
                result,
                Err(Error::InvalidGeometry {
                    parameter: "nyquist_velocity_mps",
                    ..
                })
            ),
            "Nyquist velocity {value} should be rejected"
        );

        let result = Radial::builder(1, 10.0, 1, 0.5)
            .unambiguous_range_km(value)
            .build();
        assert!(
            matches!(
                result,
                Err(Error::InvalidGeometry {
                    parameter: "unambiguous_range_km",
                    ..
                })
            ),
            "unambiguous range {value} should be rejected"
        );
    }
}

fn cut(elevation_angle_degrees: f32, waveform_type: WaveformType) -> ElevationCut {
    ElevationCut::builder(
        elevation_angle_degrees,
        waveform_type,
        ChannelConfiguration::ConstantPhase,
    )
    .build()
}

fn sweep(elevation_number: u8, elevation_angle_degrees: f32) -> Sweep {
    let radial = Radial::builder(1, 0.0, elevation_number, elevation_angle_degrees)
        .build()
        .unwrap();
    Sweep::new(elevation_number, vec![radial]).unwrap()
}

/// A split-cut scan: surveillance and Doppler cuts at 0.5 and 1.5 degrees, then a batch cut.
fn split_cut_scan() -> Scan {
    let vcp = VolumeCoveragePattern::builder(VcpNumber::VCP_212)
        .elevation_cuts(vec![
            cut(0.5, WaveformType::ContiguousSurveillance),
            cut(
                0.5,
                WaveformType::ContiguousDopplerWithoutAmbiguityResolution,
            ),
            cut(1.5, WaveformType::ContiguousSurveillance),
            cut(
                1.5,
                WaveformType::ContiguousDopplerWithoutAmbiguityResolution,
            ),
            cut(2.4, WaveformType::Batch),
        ])
        .build();
    let sweeps = vec![
        sweep(1, 0.5),
        sweep(2, 0.5),
        sweep(3, 1.5),
        sweep(4, 1.5),
        sweep(5, 2.4),
        sweep(6, 3.4),
    ];
    Scan::new(vcp, sweeps)
}

#[test]
fn elevation_cut_matches_sweep_elevation_number() {
    let scan = split_cut_scan();

    let surveillance = scan.elevation_cut(&scan.sweeps()[0]).unwrap();
    assert_eq!(
        surveillance.waveform_type(),
        WaveformType::ContiguousSurveillance
    );
    let doppler = scan.elevation_cut(&scan.sweeps()[3]).unwrap();
    assert_eq!(
        doppler.waveform_type(),
        WaveformType::ContiguousDopplerWithoutAmbiguityResolution
    );
    assert_eq!(doppler.elevation_angle_degrees(), 1.5);
    let batch = scan.elevation_cut(&scan.sweeps()[4]).unwrap();
    assert_eq!(batch.waveform_type(), WaveformType::Batch);
}

#[test]
fn elevation_cut_is_none_out_of_range() {
    let scan = split_cut_scan();
    assert!(scan.elevation_cut(&scan.sweeps()[5]).is_none());
    assert!(scan.elevation_cut(&sweep(0, 0.5)).is_none());
    assert!(scan.elevation_cut(&sweep(255, 0.5)).is_none());
}

#[test]
fn sweeps_with_waveform_selects_by_cut_in_scan_order() {
    let scan = split_cut_scan();

    let numbers = |waveform_type| {
        scan.sweeps_with_waveform(waveform_type)
            .map(Sweep::elevation_number)
            .collect::<Vec<_>>()
    };
    assert_eq!(numbers(WaveformType::ContiguousSurveillance), vec![1, 3]);
    assert_eq!(
        numbers(WaveformType::ContiguousDopplerWithoutAmbiguityResolution),
        vec![2, 4]
    );
    assert_eq!(numbers(WaveformType::Batch), vec![5]);
    assert!(numbers(WaveformType::StaggeredPulsePair).is_empty());
}

#[test]
fn vcp_number_constants_carry_their_numbers() {
    for (number, vcp) in KNOWN_VCPS {
        assert_eq!(VcpNumber::new(number), vcp);
        assert_eq!(vcp.number(), number);
        assert_eq!(u16::from(vcp), number);
        assert_eq!(vcp.to_string(), format!("VCP {number}"));
    }

    assert_eq!(VcpNumber::from(999), VcpNumber::new(999));
    assert_eq!(VcpNumber::new(999).number(), 999);
    assert_eq!(VcpNumber::new(999).to_string(), "VCP 999");
}

fn coordinate_system() -> RadarCoordinateSystem {
    let site = Site::builder(*b"KTLX", 35.3331, -97.2778)
        .site_elevation_meters(370)
        .feedhorn_height_meters(20)
        .build()
        .unwrap();
    RadarCoordinateSystem::new(&site)
}

/// Distance in meters between two nearby points on the spherical earth.
fn surface_distance_m(a: GeoPoint, b: GeoPoint) -> f64 {
    let (lat1, lat2) = (a.latitude.to_radians(), b.latitude.to_radians());
    let d_lat = lat2 - lat1;
    let d_lon = (b.longitude - a.longitude).to_radians();
    let h = (d_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
    2.0 * h.sqrt().asin() * 6_371_000.0
}

#[test]
fn geolocation_round_trips_at_long_range_and_steep_tilts() {
    let coords = coordinate_system();

    for elevation_angle_degrees in [0.5, 1.5, 4.0, 10.0, 19.5] {
        for range_km in [1.0, 50.0, 150.0, 230.0, 300.0] {
            for azimuth_angle_degrees in [0.0, 45.0, 200.0, 315.0] {
                let polar = PolarPoint {
                    azimuth_angle_degrees,
                    range_km,
                    elevation_angle_degrees,
                };
                let geo = coords.polar_to_geo(polar);
                let point = GeoPoint {
                    latitude: geo.latitude,
                    longitude: geo.longitude,
                };
                let back = coords.geo_to_polar(point, elevation_angle_degrees);

                let range_error_m = (back.range_km - range_km).abs() * 1000.0;
                assert!(
                    range_error_m < 1.0,
                    "range error {range_error_m} m at {range_km} km, {elevation_angle_degrees} deg"
                );

                let again = coords.polar_to_geo(back);
                let position_error_m = surface_distance_m(
                    point,
                    GeoPoint {
                        latitude: again.latitude,
                        longitude: again.longitude,
                    },
                );
                assert!(
                    position_error_m < 1.0,
                    "position error {position_error_m} m at {range_km} km, \
                     {elevation_angle_degrees} deg"
                );
                let height_error_m = (again.altitude_meters - geo.altitude_meters).abs();
                assert!(
                    height_error_m < 1.0,
                    "height error {height_error_m} m at {range_km} km, \
                     {elevation_angle_degrees} deg"
                );
            }
        }
    }
}

#[test]
fn polar_to_geo_uses_beam_height_and_ground_range() {
    let coords = coordinate_system();
    let geo = coords.polar_to_geo(PolarPoint {
        azimuth_angle_degrees: 0.0,
        range_km: 300.0,
        elevation_angle_degrees: 19.5,
    });
    assert_eq!(geo.altitude_meters, coords.beam_height_meters(300.0, 19.5));

    let north = surface_distance_m(
        GeoPoint {
            latitude: coords.latitude(),
            longitude: coords.longitude(),
        },
        GeoPoint {
            latitude: geo.latitude,
            longitude: geo.longitude,
        },
    );
    assert!((north / 1000.0 - coords.ground_range_km(300.0, 19.5)).abs() < 0.001);
}

#[test]
fn beam_height_matches_textbook_value() {
    let coords = coordinate_system();

    // h = sqrt(r^2 + R'^2 + 2 r R' sin(theta)) - R' with R' = 4/3 * 6371 km is about 1461 m at
    // 100 km on a 0.5 degree tilt, above the antenna.
    let above_antenna = coords.beam_height_meters(100.0, 0.5) - coords.antenna_height_meters();
    assert!(
        (above_antenna - 1461.0).abs() < 5.0,
        "beam height {above_antenna} m above the antenna"
    );

    assert!((coords.beam_height_meters(0.0, 0.5) - coords.antenna_height_meters()).abs() < 1e-6);
    assert!(coords.ground_range_km(100.0, 0.5) < 100.0);
    assert!(coords.ground_range_km(100.0, 0.5) > 99.9);
}

#[test]
fn nearest_site_wraps_across_the_antimeridian() {
    // West of 180 degrees in the Bering Sea, the nearest radar is Nome across the antimeridian,
    // not Guam.
    let site = registry::nearest_site(60.0, 179.5).unwrap();
    assert_eq!(site.id, "PAEC");

    let site = registry::nearest_site(60.0, -179.5).unwrap();
    assert_eq!(site.id, "PAEC");
}
