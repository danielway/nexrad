//! Integration tests for unified error type conversions.
//!
//! These tests verify that errors from all sub-crates properly convert to the
//! unified `nexrad::Error` type through the `From` trait implementations.

#[cfg(feature = "decode")]
use std::error::Error as StdError;

#[cfg(feature = "model")]
#[test]
fn test_model_error_conversion() {
    // Create a model error
    let model_err = nexrad_model::result::Error::ElevationMismatchError;

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
    let decode_err = nexrad_decode::result::Error::MessageMissingDateError;

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
    let decode_err =
        nexrad_decode::result::Error::DecodingError("invalid magic number".to_string());

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
    let data_err = nexrad_data::result::Error::CompressedDataError;

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

#[cfg(all(feature = "data", feature = "decode"))]
#[test]
fn test_nested_decode_error_through_data() {
    // Create a decode error and wrap it in a data error
    let decode_err = nexrad_decode::result::Error::UnexpectedEof;
    let data_err: nexrad_data::result::Error = decode_err.into();

    // Convert to unified error
    let unified_err: nexrad::Error = data_err.into();

    // Verify it's wrapped as a Data error (since it came through nexrad-data)
    match unified_err {
        nexrad::Error::Data(_) => {}
        #[allow(unreachable_patterns)]
        _ => panic!("Expected Error::Data variant"),
    }

    // Verify the error chain is preserved
    let err_string = unified_err.to_string();
    assert!(
        err_string.contains("data error"),
        "Expected 'data error' in message, got: {}",
        err_string
    );
}

#[cfg(feature = "decode")]
#[test]
fn test_io_error_conversion_through_decode() {
    // Create an I/O error and convert through decode
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let decode_err: nexrad_decode::result::Error = io_err.into();

    // Convert to unified error
    let unified_err: nexrad::Error = decode_err.into();

    // Verify the error message
    let err_string = unified_err.to_string();
    assert!(
        err_string.contains("decode error"),
        "Expected 'decode error' in message, got: {}",
        err_string
    );
}

#[cfg(feature = "data")]
#[test]
fn test_io_error_conversion_through_data() {
    // Create an I/O error and convert through data
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let data_err: nexrad_data::result::Error = io_err.into();

    // Convert to unified error
    let unified_err: nexrad::Error = data_err.into();

    // Verify the error message
    let err_string = unified_err.to_string();
    assert!(
        err_string.contains("data error"),
        "Expected 'data error' in message, got: {}",
        err_string
    );
}

#[cfg(feature = "decode")]
#[test]
fn test_error_source_chain() {
    // Create a decode error
    let decode_err = nexrad_decode::result::Error::DecodingError("test error".to_string());

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
    let model_err = nexrad_model::result::Error::ElevationMismatchError;
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
    let render_err = nexrad_render::result::Error::ProductNotFound;

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

    let model_err = nexrad_model::result::Error::ElevationMismatchError;
    let _: nexrad::Error = model_err.into();

    let decode_err = nexrad_decode::result::Error::UnexpectedEof;
    let _: nexrad::Error = decode_err.into();

    let data_err = nexrad_data::result::Error::CompressedDataError;
    let _: nexrad::Error = data_err.into();
}

#[cfg(all(
    feature = "model",
    feature = "decode",
    feature = "data",
    feature = "render"
))]
#[test]
fn test_all_features_including_render() {
    // This test verifies all error variants including render
    let model_err = nexrad_model::result::Error::ElevationMismatchError;
    let _: nexrad::Error = model_err.into();

    let decode_err = nexrad_decode::result::Error::UnexpectedEof;
    let _: nexrad::Error = decode_err.into();

    let data_err = nexrad_data::result::Error::CompressedDataError;
    let _: nexrad::Error = data_err.into();

    let render_err = nexrad_render::result::Error::ProductNotFound;
    let _: nexrad::Error = render_err.into();
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
