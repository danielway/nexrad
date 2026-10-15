use crate::result::Result;
use crate::SweepProcessor;
use nexrad_model::data::SweepField;
use std::fmt;

/// A composable pipeline of sweep processing steps.
///
/// Steps are executed in order, with the output of each step becoming the input
/// to the next. The pipeline itself implements [`SweepProcessor`], so pipelines
/// can be nested.
///
/// # Example
///
/// ```
/// use nexrad_process::{SweepPipeline, filter::ThresholdFilter};
/// # let input_field = nexrad_model::data::SweepField::builder("Reflectivity", "dBZ")
/// #     .product(nexrad_model::data::Product::Reflectivity)
/// #     .elevation_angle_degrees(0.5)
/// #     .azimuth_angles_degrees((0..360).map(|a| a as f32).collect())
/// #     .first_gate_range_km(2.0)
/// #     .gate_interval_km(0.25)
/// #     .gate_count(400)
/// #     .build()?;
///
/// let pipeline = SweepPipeline::new()
///     .then(ThresholdFilter::new(Some(5.0), None)?)
///     .then(ThresholdFilter::new(None, Some(75.0))?);
///
/// let output = pipeline.execute(&input_field)?;
/// # Ok::<(), nexrad_process::Error>(())
/// ```
pub struct SweepPipeline {
    steps: Vec<Box<dyn SweepProcessor>>,
}

impl SweepPipeline {
    /// Create a new empty pipeline.
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    /// Append a processing step to the pipeline.
    pub fn then(mut self, processor: impl SweepProcessor + 'static) -> Self {
        self.steps.push(Box::new(processor));
        self
    }

    /// Number of steps in the pipeline.
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Whether the pipeline has no steps.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// The pipeline's steps, in execution order.
    pub fn steps(&self) -> impl Iterator<Item = &dyn SweepProcessor> {
        self.steps
            .iter()
            .map(|step| step.as_ref() as &dyn SweepProcessor)
    }

    /// Execute the pipeline, returning the final processed field.
    ///
    /// If the pipeline has no steps, the input is cloned and returned as-is.
    ///
    /// # Errors
    ///
    /// Returns the error of the first step that fails.
    pub fn execute(&self, input: &SweepField) -> Result<SweepField> {
        if self.steps.is_empty() {
            return Ok(input.clone());
        }

        let mut current = self.steps[0].process(input)?;
        for step in &self.steps[1..] {
            current = step.process(&current)?;
        }
        Ok(current)
    }
}

impl fmt::Debug for SweepPipeline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<&str> = self.steps().map(|step| step.name()).collect();
        f.debug_struct("SweepPipeline")
            .field("steps", &names)
            .finish()
    }
}

impl Default for SweepPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl SweepProcessor for SweepPipeline {
    fn name(&self) -> &str {
        "Pipeline"
    }

    fn process(&self, input: &SweepField) -> Result<SweepField> {
        self.execute(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::derived::{CompositeReflectivity, StormRelativeVelocity, VerticalCrossSection};
    use crate::detection::StormCellDetector;
    use crate::filter::{
        CorrelationCoefficientFilter, GaussianSmooth, MedianFilter, ThresholdFilter,
    };
    use crate::CartesianGrid;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn test_public_types_are_send_and_sync() {
        assert_send_sync::<SweepPipeline>();
        assert_send_sync::<ThresholdFilter>();
        assert_send_sync::<MedianFilter>();
        assert_send_sync::<GaussianSmooth>();
        assert_send_sync::<CorrelationCoefficientFilter>();
        assert_send_sync::<StormRelativeVelocity>();
        assert_send_sync::<CompositeReflectivity>();
        assert_send_sync::<VerticalCrossSection>();
        assert_send_sync::<StormCellDetector>();
        assert_send_sync::<CartesianGrid>();
        assert_send_sync::<Box<dyn SweepProcessor>>();
        assert_send_sync::<Box<dyn crate::ScanDerivedProduct>>();
    }

    #[test]
    fn test_introspection() -> Result<()> {
        let empty = SweepPipeline::new();
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
        assert_eq!(empty.steps().count(), 0);

        let pipeline = SweepPipeline::new()
            .then(ThresholdFilter::new(Some(5.0), None)?)
            .then(MedianFilter::new(3, 3)?);
        assert!(!pipeline.is_empty());
        assert_eq!(pipeline.len(), 2);
        let names: Vec<&str> = pipeline.steps().map(|step| step.name()).collect();
        assert_eq!(names, ["ThresholdFilter", "MedianFilter"]);
        Ok(())
    }

    #[test]
    fn test_debug_lists_step_names() -> Result<()> {
        let pipeline = SweepPipeline::new()
            .then(ThresholdFilter::new(Some(5.0), None)?)
            .then(GaussianSmooth::new(1.0, 1.0)?);

        assert_eq!(
            format!("{pipeline:?}"),
            r#"SweepPipeline { steps: ["ThresholdFilter", "GaussianSmooth"] }"#
        );
        Ok(())
    }

    #[test]
    fn test_debug_empty_pipeline() {
        assert_eq!(
            format!("{:?}", SweepPipeline::new()),
            "SweepPipeline { steps: [] }"
        );
    }
}
