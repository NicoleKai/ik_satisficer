use std::{default, marker::PhantomData, sync::Arc};

use bevy_math::Vec3;
use derive_new::new;

#[derive(Debug, Clone)]
enum LimbComponent {
    joint(Joint),
    segment(Segment),
}

#[derive(Debug, Clone)]
pub struct Segment {
    segment_name: Arc<str>,
    position: Vec3,
    length: f32,
    joint: Joint,
}

#[derive(Debug, Clone)]
pub struct Joint {
    position: Vec3,
}

// impl Segment {
//     pub fn new(segment_name: impl Into<Arc<str>>, position: Vec3) -> Self {
//         Self {
//             segment_name: segment_name.into(),
//             joint: Joint { position },
//         }
//     }
// }

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
    components: Vec<LimbComponent>,
}

enum MathState {
    Initialization,
    Bouncing,
    BounceBack,
}

impl Limb {
    pub fn new(bounce_iterations: usize, joints: Vec<LimbComponent>) -> Self {
        let len = joints.len();
        Self {
            // TODO: bounce iterations can be changed dynamically
            bounce_iterations: 1,
            iterations: compute_iterations(bounce_iterations, len),
            components: joints,
        }
    }
    pub fn satisfice(&mut self, target_end_effector: Vec3) {
        // TODO: handle niche cases, e.g., joint count <= 2
        let v_len = self.components.len();
        let iterations = (v_len - 1) * 2 + 1;
        dbg!(&v_len);
        if v_len <= 0 {
            return;
        }
        let mut index = v_len - 1;
        let mut math_state = MathState::Initialization;
        let mut bounce_state = BounceState::Forward;
        for _bounce_counter in 0..self.bounce_iterations {
            
            // (*self.components.last_mut().expect("crabs attacking")).position = target_end_effector;
            // // dbg!("END EFFECTOR", &self.joints.);

            // for _counter in 0..iterations {
            //     if index >= v_len {
            //         bounce_state = BounceState::Backward;
            //         // Will break for lengths less than 2 obvs
            //         index = v_len - 2;
            //     }
            //     if index <= 0 {
            //         bounce_state = BounceState::Forward;
            //     }
            //     let mut ptr_current = self.components.get(index).unwrap();
            //     let mut ptr_next = self.components.get(index + 1).unwrap();
            //     let diff = ptr_next.position - ptr_current.position;
            //     let pos = (ptr_next.position - ptr_current.position).normalize();
            //     match bounce_state {
            //         BounceState::Forward => index += 1,
            //         BounceState::Backward => index -= 1,
            //     }
            }
        }
    }
}

// #[cfg(test)]
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
