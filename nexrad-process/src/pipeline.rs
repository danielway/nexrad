use crate::result::Result;
use crate::SweepProcessor;
use nexrad_model::data::SweepField;

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
///     .then(ThresholdFilter { min: Some(5.0), max: None })
///     .then(ThresholdFilter { min: None, max: Some(75.0) });
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
