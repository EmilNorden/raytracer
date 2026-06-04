// Cumulative distribution function
pub struct CDF {
    cumulative_weights: Vec<f32>,
    total_weight: f32,
}

impl CDF {
    pub fn new_from_iterator<I, T>(items: I, weight_callback: fn(&T) -> f32) -> Self
    where I: IntoIterator<Item = T> {
        let mut cumulative_weights = Vec::new();
        let mut total_weight = 0.0;
        for item in items {
            let weight = weight_callback(&item);
            total_weight += weight;
            cumulative_weights.push(total_weight);
        }
        Self {
            cumulative_weights,
            total_weight,
        }
    }

    pub fn sample(&self, rng: &mut impl rand::Rng) -> usize {
        let x = rng.random::<f32>() * self.total_weight;
        self.cumulative_weights.partition_point(|&v| v <= x)
    }

    pub fn total_weight(&self) -> f32 { self.total_weight }
}