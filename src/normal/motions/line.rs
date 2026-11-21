use crate::{
    editor::buffer::Buffer,
    normal::{
        resolver::{ResolveError, ResolveResult},
        scanner::{is_blank, line_chars},
        target::{Inclusivity, MotionOutput, Position},
    },
};

fn last_line(buffer: &Buffer) -> ResolveResult<usize> {
    buffer
        .line_count()
        .checked_sub(1)
        .ok_or(ResolveError::InvalidPosition)
}

fn chars_of(buffer: &Buffer, line: usize) -> ResolveResult<Vec<char>> {
    line_chars(buffer, line).ok_or(ResolveError::InvalidPosition)
}

fn lines_down(buffer: &Buffer, line: usize, n: usize) -> ResolveResult<usize> {
    if n == 0 {
        return Ok(line);
    }

    let last = last_line(buffer)?;

    if line >= last {
        return Err(ResolveError::Failed);
    }

    Ok((line + n).min(last))
}

pub fn first_non_blank_col(chars: &[char]) -> usize {
    chars
        .iter()
        .position(|c| !is_blank(*c))
        .unwrap_or(chars.len().saturating_sub(1))
}

fn last_non_blank_col(chars: &[char]) -> usize {
    chars.iter().rposition(|c| !is_blank(*c)).unwrap_or(0)
}

pub fn first_non_blank_on(buffer: &Buffer, line: usize) -> ResolveResult<MotionOutput> {
    let chars = chars_of(buffer, line)?;

    Ok(MotionOutput::lines(Position::new(
        line,
        first_non_blank_col(&chars),
    )))
}

pub fn start(position: Position) -> MotionOutput {
    MotionOutput::chars(Position::new(position.line, 0), Inclusivity::Exclusive)
}

pub fn first_non_blank(buffer: &Buffer, position: Position) -> ResolveResult<MotionOutput> {
    let chars = chars_of(buffer, position.line)?;

    Ok(MotionOutput::chars(
        Position::new(position.line, first_non_blank_col(&chars)),
        Inclusivity::Exclusive,
    ))
}

pub fn end(buffer: &Buffer, position: Position, count: usize) -> ResolveResult<MotionOutput> {
    let line = lines_down(buffer, position.line, count - 1)?;
    let len = chars_of(buffer, line)?.len();

    Ok(MotionOutput::chars(
        Position::new(line, len.saturating_sub(1)),
        Inclusivity::Inclusive,
    ))
}

pub fn last_non_blank(
    buffer: &Buffer,
    position: Position,
    count: usize,
) -> ResolveResult<MotionOutput> {
    let line = lines_down(buffer, position.line, count - 1)?;
    let chars = chars_of(buffer, line)?;

    Ok(MotionOutput::chars(
        Position::new(line, last_non_blank_col(&chars)),
        Inclusivity::Inclusive,
    ))
}

pub fn column(buffer: &Buffer, position: Position, count: usize) -> ResolveResult<MotionOutput> {
    let len = chars_of(buffer, position.line)?.len();

    Ok(MotionOutput::chars(
        Position::new(position.line, (count - 1).min(len.saturating_sub(1))),
        Inclusivity::Exclusive,
    ))
}

pub fn next_line(buffer: &Buffer, position: Position, count: usize) -> ResolveResult<MotionOutput> {
    let line = position.line + count;

    if line > last_line(buffer)? {
        return Err(ResolveError::Failed);
    }

    first_non_blank_on(buffer, line)
}

pub fn prev_line(buffer: &Buffer, position: Position, count: usize) -> ResolveResult<MotionOutput> {
    let line = position
        .line
        .checked_sub(count)
        .ok_or(ResolveError::Failed)?;

    first_non_blank_on(buffer, line)
}

pub fn current_line(
    buffer: &Buffer,
    position: Position,
    count: usize,
) -> ResolveResult<MotionOutput> {
    let line = (position.line + count - 1).min(last_line(buffer)?);

    first_non_blank_on(buffer, line)
}
