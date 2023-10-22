use std::{default, sync::Arc};

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

/// Last item in Vec is end effector
#[derive(new, Default, Debug, Clone)]
pub struct Limb {
    joints: Vec<Joint>,
}

impl Limb {
    pub fn iter_mut(&mut self) -> IterLimb {
        IterLimb {
            count: 0,
            bounce: 1,
            collection: self,
            mode: IterLimbMode::Backward,
        }
    }
    pub fn satisfice(&mut self, target_end_effector: Vec3) {
        // TODO: zero-copy
        for value in self.iter_mut() {}
        // let mut r = self.joints.clone();
        // r.reverse();
        // r.remove(r.len() - 1);
        // let mut joints = r.iter_mut().chain(self.joints.iter_mut()).into_iter();
        // while let Some(mut joint) = joints.next() {
        //     *joint = Joint::new("pls", Vec3::ZERO);
        //     // joint.peek(
        //     // match joints {
        //     //     &[prev_joint, curr_joint, next_joint] => {
        //     //         dbg!(curr_joint);
        //     //     }
        //     //     _ => panic!("crabs attacking!11"),
        //     // }
        // }
    }
}

pub enum IterLimbMode {
    Forward,
    Backward,
}

struct MyStruct {
    data: Vec<i32>,
}

impl MyStruct {
    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut i32> {
        self.data.iter_mut()
    }
}

pub struct IterLimb<'a> {
    count: usize,
    bounce: usize,
    collection: *mut Limb,
    mode: IterLimbMode,
    _marker: std::marker::PhantomData<&'a mut Limb>,
}

impl<'a> Iterator for IterLimb<'a> {
    type Item = &'a mut Joint;

    fn next(&mut self) -> Option<Self::Item> {
        // if self.count > self.collection.joints.len() {}
        let count = self.count;
        unsafe { *self.collection }.joints.get_mut(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // let result = add(2, 2);
        // assert_eq!(result, 4);
        let mut limb = Limb::default();
        limb.joints
            .push(Joint::new("ground", Vec3::new(0., 0., 0.)));
        limb.joints
            .push(Joint::new("first joint", Vec3::new(1., 2., 3.)));
        limb.joints
            .push(Joint::new("second joint", Vec3::new(4., 5., 6.)));
        limb.joints
            .push(Joint::new("end effector", Vec3::new(7., 8., 9.)));

        limb.satisfice(Vec3::ZERO);

        assert!(false);
    }
}
