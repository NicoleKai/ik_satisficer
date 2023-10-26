use bevy_math::Vec3;
use snafu::{OptionExt, Snafu};

#[derive(Debug, Default)]
struct Segment {
    start: Vec3,
    end: Vec3,
    length: f32,
}

#[derive(Debug, Default)]
struct Limb {
    segments: Vec<Segment>,
    target: Vec3,
    bounces: usize,
}

#[derive(Debug, Snafu)]
enum SolveError {
    EmptyLimb,
}

impl Limb {
    fn new(n_segments: usize, bounces: usize, target: Vec3) -> Self {
        let mut segments = Vec::new();
        for i in 0..n_segments {
            segments.push(Segment {
                start: Vec3::Y * i as f32,
                end: Vec3::Y * (i + 1) as f32,
                length: 1.0,
            });
        }
        Self {
            segments,
            target,
            bounces,
        }
    }
    fn solve(&mut self) -> Result<(), SolveError> {
        let len = self.segments.len();
        for _bounce in 0..self.bounces {
            // Set end effector to target
            (*self.segments.last_mut().context(EmptyLimbSnafu)?).end = self.target;
            // iterates from len (max index) to zero
            for index in len..0 {
                // get current segment
                let current = &mut self.segments[index];
                // compute o-hat (current start minus current end) then normalize
                let o_hat = (current.start - current.end).normalize();
                // set current end to o_hat scaled by length of current segment
                current.end = o_hat * current.length;
                //set next end to current start
                let current_start = current.start.clone();
                if let Some(ref mut next) = self.segments.get_mut(index + 1) {
                    next.end = current_start;
                }
            }
            // iterates from 0 to len (max index)
            for index in 0..len {
                // get current segment
                let current = &mut self.segments[index];
                // calculate o_hat (current end minus current start, then normalized)
                let o_hat = (current.end - current.start).normalize();
                // current end is set to o_hat scaled by current length
                current.end = o_hat * current.length;
                // set next start to current end
                let current_end = current.end.clone();
                if let Some(ref mut next) = self.segments.get_mut(index + 1) {
                    next.start = current_end;
                }
            }
        }
        Ok(())
    }
}

fn main() {
    let mut l = Limb::new(3, 1, Vec3::ZERO);
    dbg!(&l);
    l.solve().unwrap();
    dbg!(&l);
}
