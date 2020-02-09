pub fn lerp() {
    unimplemented!();
}

// TODO: Use a generic array for user defined length
pub struct MovingAverage {
    queue: Vec<f64>,
    index: usize,
}

impl MovingAverage {
    pub fn new(array_size: usize, initial_value: Option<f64>) -> MovingAverage {
        let size = match array_size {
            0 => 1,
            n => n,
        };
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

#[cfg(test)]
mod tests {
    use super::*;
    use approx::*;

    #[test]
    fn test_moving_average() {
        // A vec! of len() 0 is invalid, so we return one with len() 1
        let result = MovingAverage::new(0, None);
        assert_eq!(result.queue.len(), 1);

        let result = MovingAverage::new(5, Some(1.0));
        // The vec! should have 5 elements
        assert_eq!(result.queue.len(), 5);
        // All values should be 1.0f64
        for n in result.queue.iter() {
            assert_abs_diff_eq!(*n, 1.0);
        }

        let mut result = MovingAverage::new(2, Some(0.0));
        // The average of 5.0 and 10.0 should equal 7.5
        result.add(5.0);
        result.add(10.0);
        assert_abs_diff_eq!(result.get_avg(), 7.5 as f64);

        // The average of 5.0 should equal 2.5
        result.add(2.5);
        result.add(2.5);
        assert_abs_diff_eq!(result.get_avg(), 2.5 as f64);
    }
}
