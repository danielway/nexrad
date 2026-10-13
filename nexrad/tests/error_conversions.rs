//! Integration tests for unified error type conversions.
//!
//! These tests verify that errors from all sub-crates properly convert to the
//! unified `nexrad::Error` type through the `From` trait implementations.

#[cfg(feature = "decode")]
use std::error::Error as StdError;

#[cfg(feature = "aws")]
#[test]
fn test_no_data_available_message() {
    let date = nexrad::chrono::NaiveDate::from_ymd_opt(2023, 5, 20).expect("valid date");
    let err = nexrad::Error::NoDataAvailable {
        site: "KTLX".to_string(),
        date,
    };

    assert_eq!(
        err.to_string(),
        "no data available for site KTLX on 2023-05-20"
    );
}

#[cfg(feature = "model")]
#[test]
fn test_model_error_conversion() {
    // Create a model error
    let model_err = nexrad_model::result::Error::ElevationMismatch {
        expected: 1,
        actual: 2,
    };

    // Convert to unified error
    let unified_err: nexrad::Error = model_err.into();

    // Verify it's the correct variant
    match unified_err {
        nexrad::Error::Model(_) => {}
        #[allow(unreachable_patterns)]
        _ => panic!("Expected Error::Model variant"),
    }

    // Verify the error message contains context
    let err_string = unified_err.to_string();
    assert!(
        err_string.contains("model error"),
        "Expected 'model error' in message, got: {}",
        err_string
    );
    assert!(
        err_string.contains("elevation"),
        "Expected 'elevation' in message, got: {}",
        err_string
    );
}

#[cfg(feature = "decode")]
#[test]
fn test_decode_error_conversion() {
    // Create a decode error
    let decode_err = nexrad_decode::result::Error::MessageMissingDate;

    // Convert to unified error
    let unified_err: nexrad::Error = decode_err.into();

    // Verify it's the correct variant
    match unified_err {
        nexrad::Error::Decode(_) => {}
        #[allow(unreachable_patterns)]
        _ => panic!("Expected Error::Decode variant"),
    }

    // Verify the error message contains context
    let err_string = unified_err.to_string();
    assert!(
        err_string.contains("decode error"),
        "Expected 'decode error' in message, got: {}",
        err_string
    );
}

#[cfg(feature = "decode")]
#[test]
fn test_decode_error_with_string_conversion() {
    // Create a decode error with string detail
    let decode_err = nexrad_decode::result::Error::Decoding("invalid magic number".to_string());

    // Convert to unified error
    let unified_err: nexrad::Error = decode_err.into();

    // Verify the error message preserves detail
    let err_string = unified_err.to_string();
    assert!(
        err_string.contains("decode error"),
        "Expected 'decode error' in message, got: {}",
        err_string
    );
    assert!(
        err_string.contains("invalid magic number"),
        "Expected 'invalid magic number' in message, got: {}",
        err_string
    );
}

#[cfg(feature = "data")]
#[test]
fn test_data_error_conversion() {
    // Create a data error
    let data_err = nexrad_data::result::Error::CompressedData;

    // Convert to unified error
    let unified_err: nexrad::Error = data_err.into();

    // Verify it's the correct variant
    match unified_err {
        nexrad::Error::Data(_) => {}
        #[allow(unreachable_patterns)]
        _ => panic!("Expected Error::Data variant"),
    }

    // Verify the error message contains context
    let err_string = unified_err.to_string();
    assert!(
        err_string.contains("data error"),
        "Expected 'data error' in message, got: {}",
        err_string
    );
}

#[cfg(feature = "data")]
#[test]
fn test_decode_error_through_data_is_flattened() {
    // A decode error wrapped by nexrad-data surfaces as Error::Decode
    let decode_err = nexrad_decode::result::Error::UnexpectedEof;
    let data_err: nexrad_data::result::Error = decode_err.into();

    let unified_err: nexrad::Error = data_err.into();

    match unified_err {
        nexrad::Error::Decode(nexrad_decode::result::Error::UnexpectedEof) => {}
        ref other => panic!("Expected Error::Decode(UnexpectedEof), got: {:?}", other),
    }

    let err_string = unified_err.to_string();
    assert!(
        err_string.contains("decode error"),
        "Expected 'decode error' in message, got: {}",
        err_string
    );
}

#[cfg(feature = "data")]
#[test]
fn test_model_error_through_data_is_flattened() {
    // A model error wrapped by nexrad-data surfaces as Error::Model
    let model_err = nexrad_model::result::Error::ElevationMismatch {
        expected: 1,
        actual: 2,
    };
    let data_err: nexrad_data::result::Error = model_err.into();

    let unified_err: nexrad::Error = data_err.into();

    match unified_err {
        nexrad::Error::Model(nexrad_model::result::Error::ElevationMismatch { .. }) => {}
        ref other => panic!("Expected Error::Model(ElevationMismatch), got: {:?}", other),
    }

    let err_string = unified_err.to_string();
    assert!(
        err_string.contains("model error"),
        "Expected 'model error' in message, got: {}",
        err_string
    );
}

#[cfg(feature = "data")]
#[test]
fn test_io_error_through_data_is_flattened() {
    // An I/O error wrapped by nexrad-data surfaces as Error::Io
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let data_err: nexrad_data::result::Error = io_err.into();

    let unified_err: nexrad::Error = data_err.into();

    match &unified_err {
        nexrad::Error::Io(io_err) => {
            assert_eq!(io_err.kind(), std::io::ErrorKind::PermissionDenied);
        }
        other => panic!("Expected Error::Io, got: {:?}", other),
    }

    let err_string = unified_err.to_string();
    assert!(
        err_string.contains("I/O error") && err_string.contains("access denied"),
        "Expected 'I/O error' and 'access denied' in message, got: {}",
        err_string
    );
}

#[cfg(feature = "data")]
#[test]
fn test_decode_error_through_data_keeps_source() {
    // Flattening must not lose the decode error as the source of the unified error
    let decode_err = nexrad_decode::result::Error::Decoding("test error".to_string());
    let data_err: nexrad_data::result::Error = decode_err.into();

    let unified_err: nexrad::Error = data_err.into();

    let source = unified_err
        .source()
        .expect("Expected error source to be available");
    let source_string = source.to_string();
    assert!(
        source_string.contains("test error"),
        "Expected 'test error' in source, got: {}",
        source_string
    );
}

#[cfg(feature = "data")]
#[test]
fn test_data_specific_error_keeps_source() {
    let unified_err: nexrad::Error = nexrad_data::result::Error::CompressedData.into();

    let source = unified_err
        .source()
        .expect("Expected error source to be available");
    assert!(
        source.to_string().contains("compressed data"),
        "Expected the data error as the source, got: {}",
        source
    );
}

#[cfg(feature = "decode")]
#[test]
fn test_error_source_chain() {
    // Create a decode error
    let decode_err = nexrad_decode::result::Error::Decoding("test error".to_string());

    // Convert to unified error
    let unified_err: nexrad::Error = decode_err.into();

    // Verify the source chain is accessible
    let source = unified_err.source();
    assert!(source.is_some(), "Expected error source to be available");

    // Verify the source error message
    let source_string = source.unwrap().to_string();
    assert!(
        source_string.contains("test error"),
        "Expected 'test error' in source, got: {}",
        source_string
    );
}

#[cfg(feature = "model")]
#[test]
fn test_error_debug_format() {
    // Create a model error
    let model_err = nexrad_model::result::Error::ElevationMismatch {
        expected: 1,
        actual: 2,
    };
    let unified_err: nexrad::Error = model_err.into();

    // Verify Debug format works and contains useful info
    let debug_string = format!("{:?}", unified_err);
    assert!(!debug_string.is_empty(), "Debug format should not be empty");
    assert!(
        debug_string.contains("Model"),
        "Debug format should indicate Model variant"
    );
}

#[cfg(feature = "render")]
#[test]
fn test_render_error_conversion() {
    // Create a render error
    let render_err = nexrad_render::result::Error::InvalidDimensions;

    // Convert to unified error
    let unified_err: nexrad::Error = render_err.into();

    // Verify it's the correct variant
    match unified_err {
        nexrad::Error::Render(_) => {}
        #[allow(unreachable_patterns)]
        _ => panic!("Expected Error::Render variant"),
    }

    // Verify the error message contains context
    let err_string = unified_err.to_string();
    assert!(
        err_string.contains("render error"),
        "Expected 'render error' in message, got: {}",
        err_string
    );
}

#[cfg(all(feature = "model", feature = "decode", feature = "data"))]
#[test]
fn test_all_features_enabled() {
    // This test only compiles when default features are enabled
    // It verifies that all error variants are available

    let model_err = nexrad_model::result::Error::ElevationMismatch {
        expected: 1,
        actual: 2,
    };
    let _: nexrad::Error = model_err.into();

    let decode_err = nexrad_decode::result::Error::UnexpectedEof;
    let _: nexrad::Error = decode_err.into();

    let data_err = nexrad_data::result::Error::CompressedData;
    let _: nexrad::Error = data_err.into();
}

#[cfg(all(
    feature = "model",
    feature = "decode",
    feature = "data",
    feature = "render",
    feature = "process"
))]
#[test]
fn test_all_features_including_render_and_process() {
    // This test verifies all error variants including render and process
    let model_err = nexrad_model::result::Error::ElevationMismatch {
        expected: 1,
        actual: 2,
    };
    let _: nexrad::Error = model_err.into();

    let decode_err = nexrad_decode::result::Error::UnexpectedEof;
    let _: nexrad::Error = decode_err.into();

    let data_err = nexrad_data::result::Error::CompressedData;
    let _: nexrad::Error = data_err.into();

    let render_err = nexrad_render::result::Error::InvalidDimensions;
    let _: nexrad::Error = render_err.into();

    let process_err = nexrad_process::result::Error::MissingData("test".to_string());
    let _: nexrad::Error = process_err.into();
}

#[cfg(feature = "process")]
#[test]
fn test_process_error_conversion() {
    // Create a process error
    let process_err = nexrad_process::result::Error::InvalidParameter("bad threshold".to_string());

    // Convert to unified error
    let unified_err: nexrad::Error = process_err.into();

    // Verify it's the correct variant
    match unified_err {
        nexrad::Error::Process(_) => {}
        #[allow(unreachable_patterns)]
        _ => panic!("Expected Error::Process variant"),
    }

    // Verify the error message contains context
    let err_string = unified_err.to_string();
    assert!(
        err_string.contains("process error"),
        "Expected 'process error' in message, got: {}",
        err_string
    );
    assert!(
        err_string.contains("bad threshold"),
        "Expected 'bad threshold' in message, got: {}",
        err_string
    );
}

#[test]
fn test_result_type_alias() {
    // Verify that the Result type alias works as expected
    fn returns_result() -> nexrad::Result<()> {
        Ok(())
    }

    assert!(returns_result().is_ok());
}

#[cfg(feature = "decode")]
#[test]
fn test_question_mark_operator() {
    // Verify that the ? operator works with automatic conversion
    fn inner() -> nexrad_decode::result::Result<()> {
        Err(nexrad_decode::result::Error::UnexpectedEof)
    }

    fn outer() -> nexrad::Result<()> {
        inner()?;
        Ok(())
    }

    let result = outer();
    assert!(result.is_err());

    let err = result.unwrap_err();
    match err {
        nexrad::Error::Decode(_) => {}
        #[allow(unreachable_patterns)]
        _ => panic!("Expected Error::Decode variant"),
    }
}
