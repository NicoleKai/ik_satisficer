use bevy_math::Vec3;
use snafu::{OptionExt, Snafu};

#[derive(Debug, Default)]
pub struct Segment {
    pub start: Vec3,
    pub end: Vec3,
    pub length: f32,
}

#[derive(Debug, Default)]
pub struct Limb {
    pub segments: Vec<Segment>,
    pub target: Vec3,
    pub bounces: usize,
}

#[derive(Debug, Snafu)]
pub enum SolveError {
    EmptyLimb,
}

impl Limb {
    pub fn new(n_segments: usize, bounces: usize, target: Vec3) -> Self {
        let mut segments = Vec::new();
        for i in 0..n_segments {
            segments.push(Segment {
                start: Vec3::Y * i as f32,
                end: Vec3::Y * (i + 1) as f32,
                length: 1.0,
            });
        }
        // dbg!(&segments);
        Self {
            segments,
            target,
            bounces,
        }
    }
    pub fn solve(&mut self) -> Result<(), SolveError> {
        let len = self.segments.len();
        // dbg!(&len);
        for _bounce in 0..self.bounces {
            (*self.segments.last_mut().context(EmptyLimbSnafu)?).end = self.target;

            for index in (0..len).rev() {
                let current: &mut Segment = self.segments.get_mut(index).unwrap();
                (*current).end = (current.start - current.end).normalize() * current.length;
                let current_start = current.start.clone();
                if let Some(next) = self.segments.get_mut(index + 1) {
                    (*next).end = current_start;
                }
            }

            (*self.segments.first_mut().context(EmptyLimbSnafu)?).start = Vec3::ZERO;

            for index in 0..len {
                let current: &mut Segment = self.segments.get_mut(index).unwrap();
                (*current).start = (current.end - current.start).normalize() * current.length;
                let current_end = current.end.clone();
                if let Some(next) = self.segments.get_mut(index + 1) {
                    (*next).start = current_end;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test1() {
        let mut limb = Limb::new(3, 3, Vec3::splat(0.8));
        limb.solve();
        dbg!(limb);
    }
}
