use bevy_math::Vec3;

#[derive(Debug, Clone)]
pub struct FabrikChain {
    pub joints: Vec<Vec3>,
    pub lengths: Vec<f32>,
}

impl FabrikChain {
    pub fn new(joints: Vec<Vec3>) -> Self {
        let mut lengths = Vec::new();
        for i in 1..joints.len() {
            let length = joints[i].distance(joints[i - 1]);
            lengths.push(length);
        }
        Self { joints, lengths }
    }

    pub fn solve(&mut self, target: Vec3, iterations: usize) {
        for _ in 0..iterations {
            self.joints.last_mut().unwrap().clone_from(&target);
            for i in (0..self.joints.len() - 1).rev() {
                let direction = (self.joints[i] - self.joints[i + 1]).normalize();
                self.joints[i] = self.joints[i + 1] + direction * self.lengths[i];
            }

            self.joints.first_mut().unwrap().clone_from(&Vec3::ZERO);
            for i in 0..self.joints.len() - 1 {
                let direction = (self.joints[i + 1] - self.joints[i]).normalize();
                self.joints[i + 1] = self.joints[i] + direction * self.lengths[i];
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
