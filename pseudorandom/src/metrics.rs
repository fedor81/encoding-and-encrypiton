use crate::extensions::F64Ext;

pub struct TestResult {
    mean: f64,
    variance: f64,
}

pub fn test_generator<T: F64Ext>(g: &mut T, count_samples: usize) -> TestResult {
    let mut samples = Vec::with_capacity(count_samples);

    for _ in 0..count_samples {
        samples.push(g.next_f64());
    }

    TestResult::calculate(&samples)
}

impl TestResult {
    fn calculate(samples: &[f64]) -> TestResult {
        let mean = Self::calculate_mean(samples);
        let variance = Self::calculate_variance(samples, mean);
        TestResult { mean, variance }
    }

    fn calculate_mean(samples: &[f64]) -> f64 {
        samples.iter().sum::<f64>() / samples.len() as f64
    }

    fn calculate_variance(samples: &[f64], mean: f64) -> f64 {
        samples.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (samples.len() as f64 - 1.0)
    }

    pub fn mean(&self) -> f64 {
        self.mean
    }

    pub fn variance(&self) -> f64 {
        self.variance
    }
}
