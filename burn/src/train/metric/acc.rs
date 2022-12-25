use super::RunningMetricResult;
use crate::tensor::backend::Backend;
use crate::tensor::Tensor;
use crate::train::metric::{Metric, MetricStateDyn, Numeric};

pub struct AccuracyMetric {
    current: f64,
    count: usize,
    total: usize,
}

impl AccuracyMetric {
    pub fn new() -> Self {
        Self {
            count: 0,
            current: 0.0,
            total: 0,
        }
    }
}

impl Default for AccuracyMetric {
    fn default() -> Self {
        Self::new()
    }
}

impl Numeric for AccuracyMetric {
    fn value(&self) -> f64 {
        self.current * 100.0
    }
}

impl<B: Backend> Metric<(Tensor<B, 2>, Tensor<B::IntegerBackend, 1>)> for AccuracyMetric {
    fn update(&mut self, batch: &(Tensor<B, 2>, Tensor<B::IntegerBackend, 1>)) -> MetricStateDyn {
        let (outputs, targets) = batch;
        let count_current = outputs.dims()[0];

        let targets = targets.to_device(B::Device::default());
        let outputs = outputs
            .argmax(1)
            .to_device(B::Device::default())
            .reshape([count_current]);

        let total_current = outputs.equal(&targets).to_int().sum().to_data().value[0] as usize;

        self.count += count_current;
        self.total += total_current;
        self.current = total_current as f64 / count_current as f64;

        let name = String::from("Accurracy");
        let running = self.total as f64 / self.count as f64;
        let raw_running = format!("{running}");
        let raw_current = format!("{}", self.current);
        let formatted = format!(
            "running {:.2} % current {:.2} %",
            100.0 * running,
            100.0 * self.current
        );

        Box::new(RunningMetricResult {
            name,
            formatted,
            raw_running,
            raw_current,
        })
    }

    fn clear(&mut self) {
        self.count = 0;
        self.total = 0;
        self.current = 0.0;
    }
}
