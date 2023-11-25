//! Animation handling.

use chrono::Duration;
use std::ops::{AddAssign, Mul, Sub};

/// Data that can be animated.
pub struct Animated<T> {
    /// Value the data needs to get to.
    target: T,
    /// Current value of the data.
    current: T,
    /// Animation duration from begin to end.
    duration: Duration,
    /// Amount of time the animation has been going for.
    elapsed_time: Duration,
}

impl<T> Animated<T>
where
    T: Copy + PartialEq + Sub + AddAssign<<<T as Sub>::Output as Mul<f32>>::Output>,
    <T as Sub>::Output: Mul<f32>,
{
    /// Check whether the animation is complete or not.
    pub fn complete(&self) -> bool {
        self.target == self.current
    }

    /// Get the current data value.
    pub fn current(&self) -> &T {
        &self.current
    }

    /// Create a new instance of the data, with no active animation.
    pub fn new(current: T, duration: Duration) -> Self {
        Self {
            target: current,
            current,
            duration,
            elapsed_time: Duration::milliseconds(0),
        }
    }

    /// Set a new animation target value.
    pub fn set_target(&mut self, target: T) {
        self.target = target;
        self.elapsed_time = Duration::milliseconds(0);
    }

    /// Get the animation target.
    pub fn target(&self) -> &T {
        &self.target
    }

    /// Update the state of the animated data as a function of time.
    pub fn update(&mut self, elapsed: &Duration) {
        let remaining_time = self.duration - self.elapsed_time;
        // Check if the remaining animation time is less than the elapsed time given as input.
        if remaining_time <= *elapsed {
            self.current = self.target;
            self.elapsed_time = self.duration;
        } else {
            let elapsed_nanoseconds: f32 = elapsed.num_nanoseconds().unwrap_or(i64::MAX) as f32;
            let remaining_nanoseconds: f32 =
                remaining_time.num_nanoseconds().unwrap_or(i64::MAX) as f32;
            let progress_perc: f32 = elapsed_nanoseconds / remaining_nanoseconds;
            if !progress_perc.is_nan() {
                let distance = self.target - self.current;
                self.current += distance * progress_perc;
            }
            self.elapsed_time = self.elapsed_time + *elapsed;
        }
    }
}
