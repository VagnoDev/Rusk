use crate::{
    editor::buffer::Buffer,
    normal::{
        resolver::{ResolveError, ResolveResult},
        scanner::line_len,
        target::{Inclusivity, MotionOutput, Position},
    },
};

fn is_empty(buffer: &Buffer, line: usize) -> bool {
    line_len(buffer, line) == Some(0)
}

fn boundary(buffer: &Buffer, from: usize, forward: bool) -> Option<usize> {
    let last = buffer.line_count() - 1;
    let mut seen = !is_empty(buffer, from);
    let mut line = from;

    loop {
        line = if forward {
            if line >= last {
                return None;
            }
            line + 1
        } else {
            line.checked_sub(1)?
        };

        if is_empty(buffer, line) {
            if seen {
                return Some(line);
            }
        } else {
            seen = true;
        }
    }
}

pub fn resolve(
    buffer: &Buffer,
    position: Position,
    count: usize,
    forward: bool,
) -> ResolveResult<MotionOutput> {
    let last = buffer
        .line_count()
        .checked_sub(1)
        .ok_or(ResolveError::InvalidPosition)?;
    let mut current = position;

    for step in 0..count {
        match boundary(buffer, current.line, forward) {
            Some(line) => current = Position::new(line, 0),
            None => {
                let end = if forward {
                    let len = line_len(buffer, last).ok_or(ResolveError::InvalidPosition)?;
                    Position::new(last, len.saturating_sub(1))
                } else {
                    Position::new(0, 0)
                };

                if end == current {
                    if step == 0 {
                        return Err(ResolveError::Failed);
                    }
                    break;
                }

                let inclusivity = if forward {
                    Inclusivity::Inclusive
                } else {
                    Inclusivity::Exclusive
                };

                return Ok(MotionOutput::chars(end, inclusivity));
            }
        }
    }

    Ok(MotionOutput::chars(current, Inclusivity::Exclusive))
}
