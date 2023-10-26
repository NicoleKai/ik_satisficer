use std::{default, ops::DerefMut, sync::Arc};

use bevy_math::Vec3;
use derive_more::{Add, Div, From, Into, Mul, Sub};
use snafu::{OptionExt, ResultExt, Snafu};

#[derive(Debug, Clone, Snafu, PartialEq)]
enum LimbError {
    InvalidNode,
}

#[derive(Debug, Clone, Default)]
pub struct Limb(Vec<LimbNode>);

impl Limb {
    fn get_pos(&self, index: usize) -> Result<Position, LimbError> {
        match self.0.get(index) {
            Some(LimbNode::Terminus(terminus)) => Ok(terminus.pos),
            Some(LimbNode::Joint(joint)) => Ok(joint.pos),
            _ => Err(LimbError::InvalidNode),
        }
    }
    fn limb_pos(&self) -> Result<Position, LimbError> {
        match self.0.first() {
            Some(LimbNode::Terminus(terminus)) => Ok(terminus.pos),
            _ => Err(LimbError::InvalidNode),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LimbNode {
    Joint(Joint),
    Segment(Segment),
    Terminus(Terminus),
    Limb(Limb),
}

impl LimbNode {
    fn get_pos(&self) -> Option<Position> {
        match self {
            LimbNode::Joint(j) => Some(j.pos.clone()),
            LimbNode::Segment(_) => None,
            LimbNode::Terminus(t) => Some(t.pos.clone()),
            LimbNode::Limb(l) => l.limb_pos().ok(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TerminusType {
    EndEffector,
    Ground,
}

#[derive(Debug, Clone, PartialEq, From, Add, Into, Mul, Div, Sub)]
pub struct Position(Vec3);

#[derive(Debug, Clone)]
pub struct Terminus {
    terminus_type: TerminusType,
    pos: Position,
}

#[derive(Debug, Clone)]
pub struct Segment {
    length: f32,
}

#[derive(Debug, Clone)]
pub struct Joint {
    pos: Position,
}

#[derive(Debug, Clone, PartialEq, Snafu)]
pub enum SatisficeError {
    EndIsNotTerminus,
    LimbIsEmpty,
    InvalidSegment,
    InvalidLimb { source: LimbError },
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

/// Last item in Vec is end effector
#[derive(Default, Debug, Clone)]
pub struct IKSatisficer {
    bounce_iterations: usize,
    iterations: usize,
    limb: Limb,
}

pub enum MathState {
    Initialization,
    Bouncing,
    BounceBack,
}

impl IKSatisficer {
    pub fn new(bounce_iterations: usize, limb: Limb) -> Self {
        let len = limb.0.len();
        Self {
            // TODO: bounce iterations can be changed dynamically
            bounce_iterations: 1,
            iterations: compute_iterations(bounce_iterations, len),
            limb,
        }
    }
    pub fn satisfice(&mut self, target_end_effector_pos: Position) -> Result<(), SatisficeError> {
        let len = self.limb.0.len();
        for _ in 0..self.bounce_iterations {
            let mut position: Option<Position> = None;
            let mut prev_position: Option<Position> = None;
            let LimbNode::Terminus(ref mut terminus) =
                self.limb.0.last_mut().context(LimbIsEmptySnafu)?
            else {
                return Err(SatisficeError::EndIsNotTerminus);
            };

            terminus.pos = target_end_effector_pos.clone();
            for mut index in len..0 {
                self.limb.get_pos(index).context(InvalidLimbSnafu)?;

                if let Ok(new_position) = self.limb.get_pos(index) {
                    position = Some(new_position);
                }
                match (&prev_position, &position) {
                    (Some(prev_position), Some(position)) => {
                        let o_vec: Vec3 = position.0 - prev_position.0;
                        let o_vec_hat = o_vec.normalize();
                        let Some(LimbNode::Segment(prev_segment)) = self.limb.0.get(index - 1)
                        else {
                            return Err(SatisficeError::InvalidSegment);
                        };
                    }

                    (_, _) => {}
                }

                prev_position = position.clone();
            }
            for index in 0..len {
                if let Ok(new_position) = self.limb.get_pos(index) {
                    position = Some(new_position);
                }

                prev_position = position.clone();
                // match something {
                //     LimbNode::Joint(_) => todo!(),
                //     LimbNode::Segment(_) => todo!(),
                //     LimbNode::Terminus(_) => todo!(),
                // }
            }
        }

        Ok(())
        //     // TODO: handle niche cases, e.g., joint count <= 2
        //     let v_len = self.components.len();
        //     let iterations = (v_len - 1) * 2 + 1;
        //     dbg!(&v_len);
        //     if v_len <= 0 {
        //         return;
        //     }
        //     let mut index = v_len - 1;
        //     let mut math_state = MathState::Initialization;
        //     let mut bounce_state = BounceState::Forward;
        //     for _bounce_counter in 0..self.bounce_iterations {

        //         // (*self.components.last_mut().expect("crabs attacking")).position = target_end_effector;
        //         // // dbg!("END EFFECTOR", &self.joints.);

        //         // for _counter in 0..iterations {
        //         //     if index >= v_len {
        //         //         bounce_state = BounceState::Backward;
        //         //         // Will break for lengths less than 2 obvs
        //         //         index = v_len - 2;
        //         //     }
        //         //     if index <= 0 {
        //         //         bounce_state = BounceState::Forward;
        //         //     }
        //         //     let mut ptr_current = self.components.get(index).unwrap();
        //         //     let mut ptr_next = self.components.get(index + 1).unwrap();
        //         //     let diff = ptr_next.position - ptr_current.position;
        //         //     let pos = (ptr_next.position - ptr_current.position).normalize();
        //         //     match bounce_state {
        //         //         BounceState::Forward => index += 1,
        //         //         BounceState::Backward => index -= 1,
        //             // }
        //         }
        //     }
    }
}

// #[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // let result = add(2, 2);
        // assert_eq!(result, 4);
        let mut limb_vec = Vec::new();

        limb_vec.push(Terminus::Ground);
        // joints.push(Joint::new("ground", Vec3::new(0., 0., 0.)));
        // joints.push(Joint::new("first joint", Vec3::new(1., 2., 3.)));
        // joints.push(Joint::new("second joint", Vec3::new(4., 5., 6.)));
        // joints.push(Joint::new("end effector", Vec3::new(7., 8., 9.)));
        // let mut limb = Limb::new(1, joints);

        // limb.satisfice(Vec3::ZERO);

        assert!(false);
    }
}
