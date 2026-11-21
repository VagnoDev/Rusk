use crate::{
    editor::buffer::Buffer,
    normal::{
        resolver::{ResolveError, ResolveResult},
        scanner::line_len,
        target::{Inclusivity, MotionOutput, Position},
    },
};

pub fn left(position: Position, count: usize) -> ResolveResult<MotionOutput> {
    let col = position.col.saturating_sub(count);

    if col == position.col {
        return Err(ResolveError::Failed);
    }

    Ok(MotionOutput::chars(
        Position::new(position.line, col),
        Inclusivity::Exclusive,
    ))
}

pub fn right(
    buffer: &Buffer,
    position: Position,
    count: usize,
    pending: bool,
) -> ResolveResult<MotionOutput> {
    let len = line_len(buffer, position.line).ok_or(ResolveError::InvalidPosition)?;
    let limit = if pending { len } else { len.saturating_sub(1) };
    let col = position.col.saturating_add(count).min(limit);

    if col <= position.col {
        return Err(ResolveError::Failed);
    }

    Ok(MotionOutput::chars(
        Position::new(position.line, col),
        Inclusivity::Exclusive,
    ))
}
