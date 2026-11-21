use crate::{
    editor::buffer::Buffer,
    normal::{
        motion::{FindSpec, MotionState},
        resolver::{ResolveError, ResolveResult},
        scanner::line_chars,
        target::{Inclusivity, MotionOutput, Position},
    },
};

fn locate(chars: &[char], from: usize, spec: FindSpec, count: usize) -> Option<usize> {
    let mut hits = 0;

    let mut check = |col: usize| {
        if chars[col] == spec.ch {
            hits += 1;
        }
        hits == count
    };

    if spec.forward {
        (from + 1..chars.len()).find(|&col| check(col))
    } else {
        (0..from.min(chars.len())).rev().find(|&col| check(col))
    }
}

fn run(
    buffer: &Buffer,
    position: Position,
    spec: FindSpec,
    count: usize,
    repeating: bool,
) -> ResolveResult<MotionOutput> {
    let chars = line_chars(buffer, position.line).ok_or(ResolveError::InvalidPosition)?;

    let from = match (repeating && spec.till && count == 1, spec.forward) {
        (true, true) => position.col + 1,
        (true, false) => position.col.saturating_sub(1),
        (false, _) => position.col,
    };

    let hit = locate(&chars, from, spec, count).ok_or(ResolveError::Failed)?;

    let (col, inclusivity) = match (spec.forward, spec.till) {
        (true, false) => (hit, Inclusivity::Inclusive),
        (true, true) => (hit - 1, Inclusivity::Inclusive),
        (false, false) => (hit, Inclusivity::Exclusive),
        (false, true) => (hit + 1, Inclusivity::Exclusive),
    };

    Ok(MotionOutput::chars(
        Position::new(position.line, col),
        inclusivity,
    ))
}

pub fn find(
    buffer: &Buffer,
    position: Position,
    spec: FindSpec,
    count: usize,
    state: &mut MotionState,
) -> ResolveResult<MotionOutput> {
    let output = run(buffer, position, spec, count, false)?;
    state.last_find = Some(spec);
    Ok(output)
}

pub fn repeat(
    buffer: &Buffer,
    position: Position,
    count: usize,
    reverse: bool,
    state: &MotionState,
) -> ResolveResult<MotionOutput> {
    let mut spec = state.last_find.ok_or(ResolveError::Failed)?;

    if reverse {
        spec.forward = !spec.forward;
    }

    run(buffer, position, spec, count, true)
}
