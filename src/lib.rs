use std::{default, marker::PhantomData, sync::Arc};

use bevy_math::Vec3;
use derive_new::new;

#[derive(Debug, Clone)]
pub struct Joint {
    joint_name: Arc<str>,
    position: Vec3,
}

impl Joint {
    pub fn new(joint_name: impl Into<Arc<str>>, position: Vec3) -> Self {
        Self {
            joint_name: joint_name.into(),
            position,
        }
    }
}

#[inline]
fn compute_iterations(bounce_iterations: usize, length: usize) -> usize {
    bounce_iterations * (length - 1) * 2 + 1
}

enum BounceState {
    Forward,
    Backward,
}

/// Last item in Vec is end effector
#[derive(Default, Debug, Clone)]
pub struct Limb {
    bounce_iterations: usize,
    iterations: usize,
    joints: Vec<Joint>,
}

enum MathState {
    Initialization,
    Bouncing,
    BounceBack,
}

impl Limb {
    pub fn new(bounce_iterations: usize, joints: Vec<Joint>) -> Self {
        let len = joints.len();
        Self {
            // TODO: bounce iterations can be changed dynamically
            bounce_iterations: 1,
            iterations: compute_iterations(bounce_iterations, len),
            joints,
        }
    }
    pub fn satisfice(&mut self, target_end_effector: Vec3) {
        // TODO: handle niche cases, e.g., joint count <= 2
        let v_len = self.joints.len();
        dbg!(&v_len);
        if v_len <= 0 {
            return;
        }
        let mut index = v_len - 1;
        let mut math_state = MathState::Initialization;
        let mut bounce_state = BounceState::Forward;
        for _bounce_counter in 0..self.bounce_iterations {
            (*self.joints.last_mut().expect("crabs attacking")).position = target_end_effector;
            // dbg!("END EFFECTOR", &self.joints.);

            for _counter in 0..((v_len - 1) * 2 + 1) {
                if index >= v_len {
                    bounce_state = BounceState::Backward;
                    // Will break for lengths less than 2 obvs
                    index = v_len - 2;
                }
                if index <= 0 {
                    bounce_state = BounceState::Forward;
                }
                dbg!(&self.joints[index]);
                match bounce_state {
                    BounceState::Forward => index += 1,
                    BounceState::Backward => index -= 1,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // let result = add(2, 2);
        // assert_eq!(result, 4);
        let mut joints = Vec::new();
        joints.push(Joint::new("ground", Vec3::new(0., 0., 0.)));
        joints.push(Joint::new("first joint", Vec3::new(1., 2., 3.)));
        joints.push(Joint::new("second joint", Vec3::new(4., 5., 6.)));
        joints.push(Joint::new("end effector", Vec3::new(7., 8., 9.)));
        let mut limb = Limb::new(1, joints);

        limb.satisfice(Vec3::ZERO);

        assert!(false);
    }
}
