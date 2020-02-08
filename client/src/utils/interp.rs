pub fn lerp() {
    unimplemented!();
}

pub struct MovingAverage {
    data: Vec<f64>,
}

impl MovingAverage {
    pub fn new(size: usize, value: Option<f64>) -> MovingAverage {
        let value = match value {
            Some(v) => v,
            None => 0.0,
        };

        MovingAverage {
            data: vec![value; size],
        }
    }

    pub fn add(&mut self, value: f64) -> f64 {
        self.data.pop();
        self.data.insert(0, value);

        self.get()
    }

    pub fn get(&self) -> f64 {
        self.data.iter().fold(0., |sum, n| sum + n) / (self.data.len() as f64)
    }
}
