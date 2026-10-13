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
/// ```ignore
/// use nexrad_process::{SweepPipeline, filter::ThresholdFilter};
///
/// let pipeline = SweepPipeline::new()
///     .then(ThresholdFilter::new(Some(5.0), None)?)
///     .then(ThresholdFilter::new(None, Some(75.0))?);
///
/// let output = pipeline.execute(&input_field)?;
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

    /// Execute the pipeline, returning the final processed field.
    ///
    /// If the pipeline has no steps, the input is cloned and returned as-is.
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
        let names: Vec<&str> = self.steps.iter().map(|step| step.name()).collect();
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
    use crate::filter::{GaussianSmooth, ThresholdFilter};

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
