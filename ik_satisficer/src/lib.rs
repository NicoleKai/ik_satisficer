use std::{default, ops::DerefMut, sync::Arc};

use bevy_math::Vec3;
use derive_more::{Add, Div, From, Into, Mul, Sub};
use derive_new::new;
use snafu::{OptionExt, ResultExt, Snafu};

#[derive(Debug, Clone, Snafu, PartialEq)]
pub enum LimbError {
    InvalidNode,
}

#[derive(Debug, Clone, Default)]
pub struct Limb(Vec<LimbNode>);

impl Limb {
    pub fn nodes(&self) -> &Vec<LimbNode> {
        &self.0
    }
    pub fn new(num_joints: usize) -> Self {
        let mut v = Vec::new();
        v.push(LimbNode::Terminus(Terminus::new(
            TerminusType::Ground,
            Position::from(Vec3::ZERO),
        )));
        v.push(LimbNode::Segment(Segment::new(1.0)));
        for i in 0..num_joints {
            v.push(LimbNode::Joint(Joint::new(Position::from(
                Vec3::Y * i as f32,
            ))));
            v.push(LimbNode::Segment(Segment::new(1.0)));
        }
        v.push(LimbNode::Terminus(Terminus::new(
            TerminusType::EndEffector,
            Position::from(Vec3::Y * num_joints as f32),
        )));
        Self(v)
    }
}

#[derive(Debug, Clone)]
pub enum LimbNode {
    Joint(Joint),
    Segment(Segment),
    Terminus(Terminus),
    Limb(Limb),
}

#[derive(Debug, Clone)]
pub enum TerminusType {
    EndEffector,
    Ground,
}

#[derive(Debug, Clone, PartialEq, From, Add, Into, Mul, Div, Sub)]
pub struct Position(Vec3);

#[derive(new, Debug, Clone)]
pub struct Terminus {
    terminus_type: TerminusType,
    pos: Position,
}

#[derive(new, Debug, Clone)]
pub struct Segment {
    length: f32,
}

#[derive(new, Debug, Clone)]
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

pub trait Positioned {
    fn update_pos(&mut self, prev_position: &mut Option<Position>, prev_segment: &Option<Segment>) {
        match (&prev_position, prev_segment) {
            (Some(prev_position), Some(prev_segment)) => {
                let position = self.pos();
                let o_vec: Vec3 = position.0 - prev_position.0;
                let o_vec_hat = o_vec.normalize();
                self.set_pos(Position::from(o_vec_hat * prev_segment.length));
            }
            _ => {}
        }
        *prev_position = Some(self.pos());
    }
    fn pos(&self) -> Position;
    fn set_pos(&mut self, p: Position);
}

impl Positioned for Terminus {
    fn pos(&self) -> Position {
        self.pos.clone()
    }

    fn set_pos(&mut self, p: Position) {
        self.pos = p;
    }
}

impl Positioned for Joint {
    fn pos(&self) -> Position {
        self.pos.clone()
    }

    fn set_pos(&mut self, p: Position) {
        self.pos = p;
    }
}

impl Positioned for Limb {
    fn pos(&self) -> Position {
        match self.0.first() {
            Some(LimbNode::Terminus(terminus)) => terminus.pos(),
            Some(LimbNode::Joint(joint)) => joint.pos(),
            _ => panic!("Invalid limb!"),
        }
    }

    fn set_pos(&mut self, p: Position) {
        let original_pos = self.pos();
        let translation = p.clone() - original_pos;
        for v in self.0.iter_mut() {
            match v {
                LimbNode::Joint(j) => {
                    j.pos = j.pos() + translation.clone();
                }
                LimbNode::Terminus(t) => {
                    t.pos = t.pos() + translation.clone();
                }
                LimbNode::Limb(l) => l.set_pos(p.clone()),
                LimbNode::Segment(_) => {}
            }
        }
    }
}

/// Last item in Vec is end effector
#[derive(Default, Debug, Clone)]
pub struct IKSatisficer {
    bounce_iterations: usize,
    limb: Limb,
}

pub enum MathState {
    Initialization,
    Bouncing,
    BounceBack,
}

impl IKSatisficer {
    pub fn nodes(&self) -> &Vec<LimbNode> {
        &self.limb.nodes()
    }
    pub fn new(bounce_iterations: usize, limb: Limb) -> Self {
        let len = limb.0.len();
        Self {
            // TODO: bounce iterations can be changed dynamically
            bounce_iterations: 1,
            limb,
        }
    }
    fn handle_node(
        node: &mut LimbNode,
        mut prev_position: &mut Option<Position>,
        prev_segment: &mut Option<Segment>,
    ) -> Result<(), SatisficeError> {
        match node {
            LimbNode::Joint(j) => j.update_pos(&mut prev_position, &prev_segment),
            LimbNode::Segment(s) => *prev_segment = Some(s.clone()),
            LimbNode::Terminus(t) => t.update_pos(&mut prev_position, &prev_segment),
            LimbNode::Limb(l) => l.update_pos(&mut prev_position, &prev_segment),
        }
        Ok(())
    }
    pub fn satisfice(&mut self, target_end_effector_pos: Position) -> Result<(), SatisficeError> {
        let len = self.limb.0.len();
        let mut prev_position: Option<Position> = None;
        // let prev_segment: Option<Segment> = None;
        for _ in 0..self.bounce_iterations {
            let mut prev_segment: Option<Segment> = None;
            let LimbNode::Terminus(ref mut terminus) =
                self.limb.0.last_mut().context(LimbIsEmptySnafu)?
            else {
                return Err(SatisficeError::EndIsNotTerminus);
            };

            terminus.pos = target_end_effector_pos.clone();
            // bounce backward
            for index in len..0 {
                Self::handle_node(
                    self.limb.0.get_mut(index).context(LimbIsEmptySnafu)?,
                    &mut prev_position,
                    &mut prev_segment,
                )?;
            }

            // bounce forward
            for index in 0..len {
                Self::handle_node(
                    self.limb.0.get_mut(index).context(LimbIsEmptySnafu)?,
                    &mut prev_position,
                    &mut prev_segment,
                )?;
            }
        }

        Ok(())
    }
}

// #[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut limb = Limb::new(3);
        let mut ik = IKSatisficer::new(1, limb);
        ik.satisfice(Position::from(Vec3::Y * 10f32));
        dbg!(ik.limb);
        assert!(false);
    }
}
