use crate::{
    editor::buffer::Buffer,
    normal::{
        resolver::{ResolveError, ResolveResult},
        scanner::{is_blank, line_chars, line_len},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

impl Position {
    pub const fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionKind {
    Char,
    Line,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inclusivity {
    Inclusive,
    Exclusive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MotionOutput {
    pub pos: Position,
    pub kind: MotionKind,
    pub inclusivity: Inclusivity,
}

impl MotionOutput {
    pub fn chars(pos: Position, inclusivity: Inclusivity) -> Self {
        Self {
            pos,
            kind: MotionKind::Char,
            inclusivity,
        }
    }

    pub fn lines(pos: Position) -> Self {
        Self {
            pos,
            kind: MotionKind::Line,
            inclusivity: Inclusivity::Inclusive,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    pub start: Position,
    pub end: Position,
    pub kind: MotionKind,
}

impl Target {
    pub fn chars(start: Position, end: Position) -> Self {
        Self {
            start,
            end,
            kind: MotionKind::Char,
        }
    }

    pub fn lines(first: usize, last: usize) -> Self {
        Self {
            start: Position::new(first, 0),
            end: Position::new(last + 1, 0),
            kind: MotionKind::Line,
        }
    }
}

pub fn cursor_after(buffer: &Buffer, output: &MotionOutput) -> Position {
    let len = line_len(buffer, output.pos.line).unwrap_or(0);
    Position::new(output.pos.line, output.pos.col.min(len.saturating_sub(1)))
}

pub fn build_target(
    buffer: &Buffer,
    from: Position,
    output: &MotionOutput,
) -> ResolveResult<Target> {
    let (start, mut end) = if from <= output.pos {
        (from, output.pos)
    } else {
        (output.pos, from)
    };

    if output.kind == MotionKind::Line {
        return Ok(Target::lines(start.line, end.line));
    }

    let inclusive = output.inclusivity == Inclusivity::Inclusive;

    if !inclusive && end.col == 0 && end.line > start.line {
        let start_chars = line_chars(buffer, start.line).ok_or(ResolveError::InvalidPosition)?;

        if start_chars.iter().take(start.col).all(|c| is_blank(*c)) {
            return Ok(Target::lines(start.line, end.line - 1));
        }

        end.line -= 1;
        end.col = line_len(buffer, end.line).ok_or(ResolveError::InvalidPosition)?;
    }

    if inclusive {
        let len = line_len(buffer, end.line).ok_or(ResolveError::InvalidPosition)?;
        end.col = (end.col + 1).min(len);
    }

    Ok(Target::chars(start, end))
}
