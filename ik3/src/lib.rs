use std::time::{Duration, SystemTime};

use bevy_math::Vec3;

#[derive(Debug, Clone)]
pub struct FabrikChain {
    pub joints: Vec<Vec3>,
    pub lengths: Vec<f32>,
    pub angles: Vec<f32>,
    pub prev_angles: Vec<f32>,
    pub angular_velocities: Vec<f32>,
    pub prev_time: SystemTime,
}

impl FabrikChain {
    pub fn new(joints: Vec<Vec3>) -> Self {
        let mut lengths = Vec::new();
        for i in 1..joints.len() {
            let length = joints[i].distance(joints[i - 1]);
            lengths.push(length);
        }
        Self {
            joints,
            lengths,
            prev_angles: Vec::new(),
            angles: Vec::new(),
            angular_velocities: Vec::new(),
            prev_time: std::time::SystemTime::now(),
        }
    }

    pub fn solve(&mut self, target: Vec3, iterations: usize) {
        for _ in 0..iterations {
            self.joints.last_mut().unwrap().clone_from(&target);
            for i in (0..self.joints.len() - 1).rev() {
                let a = self.joints[i];
                let b = self.joints[i + 1];
                let direction = (a - b).normalize();
                self.joints[i] = b + direction * self.lengths[i];
            }

            self.joints.first_mut().unwrap().clone_from(&Vec3::ZERO);
            for i in 0..self.joints.len() - 1 {
                let a = self.joints[i];
                let b = self.joints[i + 1];
                let direction = (b - a).normalize();
                self.joints[i + 1] = a + direction * self.lengths[i];
            }
        }
        std::mem::swap(&mut self.angles, &mut self.prev_angles);
        self.angles.clear();
        for i in 2..self.joints.len() {
            let a = self.joints[i - 2];
            let b = self.joints[i - 1];
            let c = self.joints[i];
            let angle = (a - b).angle_between(c - b);
            self.angles.push(angle);
        }
        let frame_delta_time = self
            .prev_time
            .elapsed()
            .expect("Could not get elapsed time");
        self.prev_time = std::time::SystemTime::now();

        self.angular_velocities.clear();
        for i in 0..self.prev_angles.len() {
            self.angular_velocities.push(
                (self.angles[i] - self.prev_angles[i]) / (frame_delta_time.as_micros() as f32),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_lengths() {
        let joints = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
        ];
        let chain = FabrikChain::new(joints);
        dbg!(&chain);

        assert_eq!(chain.lengths, vec![1.0, 1.0]);
    }

    #[test]
    fn test_fabrik_solve() {
        let joints = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
        ];
        let mut chain = FabrikChain::new(joints);
        dbg!(&chain);
        let target = Vec3::new(3.0, 0.0, 0.0);

        chain.solve(target, 10);

        assert!((*chain.joints.last().unwrap() - target).length() < 0.01);
    }
}
