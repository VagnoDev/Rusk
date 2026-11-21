use crate::{
    editor::buffer::Buffer,
    normal::{
        motion::{Desired, MotionState},
        resolver::{ResolveError, ResolveResult},
        scanner::line_len,
        target::{MotionOutput, Position},
    },
};

pub fn resolve(
    buffer: &Buffer,
    position: Position,
    count: usize,
    down: bool,
    state: &mut MotionState,
) -> ResolveResult<MotionOutput> {
    let last = buffer.line_count().saturating_sub(1);

    let line = if down {
        if position.line >= last {
            return Err(ResolveError::Failed);
        }

        position.line.saturating_add(count).min(last)
    } else {
        if position.line == 0 {
            return Err(ResolveError::Failed);
        }

        position.line.saturating_sub(count)
    };

    let len = line_len(buffer, line).ok_or(ResolveError::InvalidPosition)?;
    let desired = state.desired.unwrap_or(Desired::Col(position.col));
    state.desired = Some(desired);

    let col = match desired {
        Desired::End => len.saturating_sub(1),
        Desired::Col(col) => col.min(len.saturating_sub(1)),
    };

    Ok(MotionOutput::lines(Position::new(line, col)))
}
