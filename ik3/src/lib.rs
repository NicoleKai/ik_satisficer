use std::{
    assert_eq,
    time::{Duration, SystemTime},
};

use bevy_math::{Mat3, Quat, Vec3};
use bevy_transform::prelude::Transform;

#[derive(Default)]
pub enum PoseDiscrepancy {
    #[default]
    WithinTolerance,
    MildDivergence,
    SevereDivergence,
    EnvironmentalCompensation,
}

type AnchorPoints = Vec<(usize, Vec3, Quat)>;
type ParentRanking = Vec<(usize, i32)>;

#[derive(Debug, Clone, Default)]
pub struct MotionHeuristics {
    pub anchor_points: AnchorPoints,
    pub parent_ranking: ParentRanking,
}

impl MotionHeuristics {
    fn new(anchor_points: AnchorPoints, parent_ranking: ParentRanking) -> Self {
        Self {
            anchor_points,
            parent_ranking,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FabrikChain {
    pub joints: Vec<Vec3>,
    pub lengths: Vec<f32>,
    pub segment_transforms: Vec<Transform>,
    pub angles: Vec<f32>,
    pub prev_angles: Vec<f32>,
    pub angular_velocities: Vec<f32>,
    pub targets: Vec<(usize, Vec3)>,
    pub motion_heuristics: MotionHeuristics,
    pub prev_time: SystemTime,
    pub lock_ground: bool,
    // FIXME: first reading computation will be way off, start with prev_time option being none, and set it to some
    // so as to skip the first computation frame
    initial_state: Option<Box<Self>>,
}

impl FabrikChain {
    pub fn new(joints: Vec<Vec3>, motion_heuristics: MotionHeuristics) -> Self {
        let mut lengths = Vec::new();
        for i in 1..joints.len() {
            let length = joints[i].distance(joints[i - 1]);
            lengths.push(length);
        }
        let new_self = Self {
            joints,
            lengths,
            prev_angles: Vec::new(),
            angles: Vec::new(),
            angular_velocities: Vec::new(),
            prev_time: std::time::SystemTime::now(),
            initial_state: None,
            segment_transforms: Vec::new(),
            motion_heuristics,
            targets: Vec::new(),
            lock_ground: true,
        };

        let mut final_self = Self {
            initial_state: Some(Box::new(new_self.clone())),
            ..new_self
        };
        final_self.recalculate();
        final_self
    }

    pub fn get_ee(&self) -> &Vec3 {
        self.joints.last().expect("Joints should not be empty")
    }

    pub fn recalculate(&mut self) {
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
        self.segment_transforms.clear();
        for i in 1..self.joints.len() {
            let a = self.joints[i];
            let b = self.joints[i - 1];

            let ab_vector = b - a;
            let ab_vector = ab_vector.normalize();

            let world_axis = Vec3::new(0.0, 1.0, 0.0); // Y-axis as an example

            // Cross to get a perpendicular vector
            let perp_vector = ab_vector.cross(world_axis).normalize();

            // Cross again to get a second perpendicular vector properly aligned
            let perp_vector2 = ab_vector.cross(perp_vector).normalize();

            // Create a quaternion from the perpendicular vector
            let quat = Quat::from_mat3(&Mat3::from_cols(ab_vector, perp_vector, perp_vector2))
                * Quat::from_rotation_z(90f32.to_radians());

            self.segment_transforms.push(Transform {
                translation: (a + b) / 2.0,
                rotation: quat,
                scale: Vec3::ONE,
            });
        }
        assert_eq!(self.segment_transforms.len(), self.lengths.len());
    }

    pub fn reset(&mut self) {
        let initial_state = self
            .initial_state
            .clone()
            .expect("initial state should not be blank");
        *self = Self {
            initial_state: Some(initial_state.clone()),
            ..*initial_state
        };
        self.recalculate();
    }

    pub fn fwd_reach(&mut self) {
        // 'FORWARD REACHING'
        for i in (0..self.joints.len() - 1).rev() {
            let a = self.joints[i];
            let b = self.joints[i + 1];
            let direction = (a - b).normalize();
            self.joints[i] = b + direction * self.lengths[i];
        }
    }

    pub fn bwd_reach(&mut self) {
        // 'BACKWARD REACHING'
        for i in 0..self.joints.len() - 1 {
            let a = self.joints[i];
            let b = self.joints[i + 1];
            let direction = (b - a).normalize();
            self.joints[i + 1] = a + direction * self.lengths[i];
        }
    }

    pub fn solve(&mut self, iterations: usize, pose_discrepancy: PoseDiscrepancy) {
        match pose_discrepancy {
            PoseDiscrepancy::WithinTolerance => {
                for _ in 0..iterations {
                    for (index, pos) in self.targets.iter() {
                        self.joints[*index] = *pos;
                    }
                    self.fwd_reach();
                    if self.lock_ground {
                        self.joints.first_mut().unwrap().clone_from(&Vec3::ZERO);
                    }
                    self.bwd_reach();
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
                self.recalculate();
            }
            PoseDiscrepancy::MildDivergence => {
                todo!(); // Adjusted solving process to correct mild discrepancies via non-linear optimization
                         // Will be using Levenberg–Marquardt algorithm
            }
            PoseDiscrepancy::SevereDivergence => {
                todo!(); // More intensive adjustments or alternative strategies
            }
            PoseDiscrepancy::EnvironmentalCompensation => {
                todo!(); // Special handling for environmental factors, introducing intentional divergences
            }
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
        let motion_heuristics = MotionHeuristics::new(Vec::new(), Vec::new());
        let chain = FabrikChain::new(joints, motion_heuristics);
        dbg!(&chain);

        assert_eq!(chain.lengths, vec![1.0, 1.0]);
    }

    // #[test]
    // fn test_fabrik_solve() {
    //     let joints = vec![
    //         Vec3::new(0.0, 0.0, 0.0),
    //         Vec3::new(1.0, 0.0, 0.0),
    //         Vec3::new(2.0, 0.0, 0.0),
    //     ];
    //     let motion_heuristics = MotionHeuristics {
    //         anchor_points: Vec::new(),
    //         parent_ranking: Vec::new(),
    //     };

    //     let mut chain = FabrikChain::new(joints, motion_heuristics);
    //     dbg!(&chain);
    //     let target = Vec3::new(3.0, 0.0, 0.0);

    //     chain.solve( 10,PoseDiscrepancy::default() );

    //     assert!((*chain.joints.last().unwrap() - target).length() < 0.01);
    // }
}
