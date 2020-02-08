use std::f32::consts::PI;

// TODO: Correct offset from the LSM303DLHC
pub fn coords_to_degrees((x, y): (f32, f32)) -> f32 {
    (y.atan2(x) * 180.0) / PI + 180.0
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Test floats properly, see https://floating-point-gui.de/errors/comparison/
    #[test]
    fn test_coords_to_degrees() {
        assert_eq!(coords_to_degrees((-1.0, 0.0)), 360.0);
        assert_eq!(coords_to_degrees((-1.0, 1.0)), 315.0);
        assert_eq!(coords_to_degrees((0.0, 1.0)), 270.0);
        assert_eq!(coords_to_degrees((1.0, 1.0)), 225.0);
        assert_eq!(coords_to_degrees((1.0, 0.0)), 180.0);
        assert_eq!(coords_to_degrees((1.0, -1.0)), 135.0);
        assert_eq!(coords_to_degrees((0.0, -1.0)), 90.0);
        assert_eq!(coords_to_degrees((-1.0, -1.0)), 45.0);
    }
}
