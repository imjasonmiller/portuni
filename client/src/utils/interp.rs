pub fn lerp() {
    unimplemented!();
}

// TODO: Turn this into a generic array and not a vec!
pub struct MovingAverage {
    queue: Vec<f64>,
    index: usize,
}

impl MovingAverage {
    pub fn new(size: usize, initial_value: Option<f64>) -> MovingAverage {
        let value = match initial_value {
            Some(v) => v,
            None => 0.0,
        };

        MovingAverage {
            queue: vec![value; size],
            index: 0,
        }
    }

    pub fn add(&mut self, value: f64) -> f64 {
        self.queue[self.index] = value;
        self.index = (self.index + 1) % self.queue.len();

        self.get_avg()
    }

    pub fn get_avg(&self) -> f64 {
        self.queue.iter().fold(0., |sum, n| sum + n) / (self.queue.len() as f64)
    }
}
