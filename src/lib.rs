//! WindMouse: A Rust implementation of the WindMouse algorithm
//!
//! This module provides a WindMouse struct and associated methods to generate
//! realistic mouse movement paths. The algorithm simulates mouse movements
//! with consideration for gravity, wind, and randomness to create more
//! human-like cursor trajectories.
//!
//! # Dependencies
//!
//! - rand = "0.9.0-alpha.2"
//!   We're using the updated syntax for future compatibility with Rust 2024.
//!   Unfortunately this requires the 0.9.0 Alpha.
//!
//! - thiserror = "1.0.63"
//!   For custom error handling.
//!
//! # Example
//!
//! ```
//! use windmouse::{WindMouse, Coordinate};
//!
//! let wind_mouse = WindMouse::new_default();
//! let start = Coordinate::new(0.0, 0.0);
//! let end = Coordinate::new(100.0, 100.0);
//! let points = wind_mouse.generate_points(start, end);
//! ```

use rand::{prelude::*, random, rng};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WindMouseError {
    #[error("Invalid wait time: min_wait ({min_wait}) must be less than or equal to max_wait ({max_wait})")]
    InvalidWaitTime { min_wait: f32, max_wait: f32 },
    #[error("Invalid parameter: {0} must be non-negative")]
    NegativeParameter(&'static str),
}

/// Represents a 2D coordinate with floating-point precision
#[derive(Clone, Copy, Debug)]
pub struct Coordinate {
    pub x: f32,
    pub y: f32,
}

impl Coordinate {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Converts the floating-point coordinate to integer values
    pub fn as_i32(&self) -> [i32; 2] {
        [self.x.round() as i32, self.y.round() as i32]
    }
}

/// WindMouse struct containing parameters for the mouse movement algorithm
///
/// This struct holds all the parameters that influence the behavior of the
/// WindMouse algorithm, allowing for fine-tuning of the generated mouse movements.
///
/// # Parameters
///
/// * `gravity`: Influences how strongly the mouse is pulled towards the target.
///   Higher values make the movement more direct.
///
/// * `wind`: Determines the amount of randomness in the mouse's path.
///   Higher values create more curved and unpredictable movements.
///
/// * `min_wait`: The minimum time (in milliseconds) to wait between mouse movements.
///   This helps simulate human-like pauses in movement.
///
/// * `max_wait`: The maximum time (in milliseconds) to wait between mouse movements.
///   Along with `min_wait`, this creates variability in movement speed.
///
/// * `max_step`: The maximum distance the mouse can move in a single step.
///   This prevents unrealistically fast movements.
///
/// * `target_area`: The distance from the end point at which the algorithm
///   starts to slow down and become more precise.
///
/// * `mouse_speed`: A general speed factor for the mouse movement.
///   Higher values result in faster overall movement.
///
/// * `random_speed`: An additional randomness factor for mouse speed.
///   This helps create more natural, variable-speed movements.
#[derive(Debug)]
pub struct WindMouse {
    pub gravity: f32,
    pub wind: f32,
    pub min_wait: f32,
    pub max_wait: f32,
    pub max_step: f32,
    pub target_area: f32,
    pub mouse_speed: f32,
    pub random_speed: f32,
}

impl WindMouse {
    /// Creates a new WindMouse instance with the specified parameters
    pub fn new(
        mouse_speed: f32,
        gravity: f32,
        wind: f32,
        min_wait: f32,
        max_wait: f32,
        max_step: f32,
        target_area: f32,
    ) -> Result<Self, WindMouseError> {
        if min_wait > max_wait {
            return Err(WindMouseError::InvalidWaitTime { min_wait, max_wait });
        }

        for &(value, name) in &[
            (mouse_speed, "mouse_speed"),
            (gravity, "gravity"),
            (wind, "wind"),
            (min_wait, "min_wait"),
            (max_wait, "max_wait"),
            (max_step, "max_step"),
            (target_area, "target_area"),
        ] {
            if value < 0.0 {
                return Err(WindMouseError::NegativeParameter(name));
            }
        }

        let random_seed = random::<f32>() * 10.0;
        let random_speed = (random_seed / 2.0 + mouse_speed / 10.0).max(0.1);

        Ok(WindMouse {
            gravity,
            wind,
            min_wait,
            max_wait,
            max_step,
            target_area,
            mouse_speed,
            random_speed,
        })
    }

    /// Creates a new WindMouse instance with default values for all parameters except target_area
    ///
    /// This function provides a convenient way to create a WindMouse instance with
    /// sensible defaults for most parameters.
    ///
    /// # Arguments
    ///
    /// * `target_area` - The distance from the end point at which the algorithm starts to slow down
    ///
    /// # Default values
    ///
    /// * `mouse_speed`: 10.0
    /// * `gravity`: 9.0
    /// * `wind`: 3.0
    /// * `min_wait`: 2.0
    /// * `max_wait`: 10.0
    /// * `max_step`: 10.0
    /// * `target_area`: 100.0
    pub fn new_default() -> Self {
        Self::new(10.0, 9.0, 3.0, 2.0, 10.0, 10.0, 100.0)
            .expect("Default values should always be valid")
    }

    /// Generates a series of points representing the mouse movement path
    /// from the start coordinate to the end coordinate
    pub fn generate_points(&self, start: Coordinate, end: Coordinate) -> Vec<[i32; 3]> {
        let mut rng = rng();
        let mut current = start;
        let mut wind_x = rng.random::<f32>() * 10.0;
        let mut wind_y = rng.random::<f32>() * 10.0;
        let mut velocity_x = 0.0;
        let mut velocity_y = 0.0;
        let wait_diff = self.max_wait - self.min_wait;
        let sqrt2 = 2.0_f32.sqrt();
        let sqrt3 = 3.0_f32.sqrt();
        let sqrt5 = 5.0_f32.sqrt();

        let mut points = Vec::new();
        let mut current_wait = 0;

        loop {
            // Calculate distance to the end point
            let dist = Self::hypot(end.x - current.x, end.y - current.y);
            if dist <= 1.0 {
                break;
            }

            // Adjust wind based on distance to target
            let wind = self.wind.min(dist);

            if dist >= self.target_area {
                // Apply wind if we're far from the target
                let w = rng.random::<f32>() * wind * 2.0 + 1.0;
                wind_x = wind_x / sqrt3 + (w - wind) / sqrt5;
                wind_y = wind_y / sqrt3 + (w - wind) / sqrt5;
            } else {
                // Reduce wind as we get closer to the target
                wind_x /= sqrt2;
                wind_y /= sqrt2;
            }

            // Update velocity based on wind and gravity
            velocity_x += wind_x;
            velocity_y += wind_y;
            velocity_x += self.gravity * (end.x - current.x) / dist;
            velocity_y += self.gravity * (end.y - current.y) / dist;

            // Normalize velocity if it exceeds max_step
            let velocity_mag = Self::hypot(velocity_x, velocity_y);
            if velocity_mag > self.max_step {
                let random_dist = self.max_step / 2.0 + rng.random::<f32>() * self.max_step / 2.0;
                velocity_x = (velocity_x / velocity_mag) * random_dist;
                velocity_y = (velocity_y / velocity_mag) * random_dist;
            }

            // Move the cursor
            let old = current;
            current.x += velocity_x;
            current.y += velocity_y;

            // Calculate wait time based on step size
            let step = Self::hypot(current.x - old.x, current.y - old.y);
            let wait = (wait_diff * (step / self.max_step) + self.min_wait).round() as i32;
            current_wait += wait;

            // Add point to the list if it's different from the previous one
            let new = Coordinate {
                x: current.x.round(),
                y: current.y.round(),
            };
            if new.as_i32() != old.as_i32() {
                points.push([new.as_i32()[0], new.as_i32()[1], current_wait]);
            }
        }

        // Ensure the end point is included
        let end_point = end.as_i32();
        if points.last().map(|&p| [p[0], p[1]]) != Some([end_point[0], end_point[1]]) {
            points.push([end_point[0], end_point[1], current_wait]);
        }

        points
    }

    /// Calculates the hypotenuse (Euclidean distance) between two points
    fn hypot(dx: f32, dy: f32) -> f32 {
        (dx * dx + dy * dy).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_valid_parameters() {
        assert!(WindMouse::new(10.0, 9.0, 3.0, 2.0, 10.0, 10.0, 100.0).is_ok());
    }

    #[test]
    fn test_new_invalid_wait_times() {
        assert!(matches!(
            WindMouse::new(10.0, 9.0, 3.0, 10.0, 2.0, 10.0, 100.0),
            Err(WindMouseError::InvalidWaitTime { .. })
        ));
    }

    #[test]
    fn test_new_negative_parameter() {
        assert!(matches!(
            WindMouse::new(-1.0, 9.0, 3.0, 2.0, 10.0, 10.0, 100.0),
            Err(WindMouseError::NegativeParameter("mouse_speed"))
        ));
    }

    #[test]
    fn test_generate_points() {
        let wind_mouse = WindMouse::new_default();
        let start = Coordinate::new(0.0, 0.0);
        let end = Coordinate::new(100.0, 100.0);
        let points = wind_mouse.generate_points(start, end);
        assert!(!points.is_empty());
        assert_eq!(points.last().unwrap()[0..2], [100, 100]);
    }
}

