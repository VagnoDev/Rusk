use crate::{
    editor::buffer::Buffer,
    normal::{
        resolver::{ResolveError, ResolveResult},
        scanner::{is_blank, line_chars},
        target::{MotionKind, Target},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Delete,
    Yank,
    Change,
    Indent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentDirection {
    Increase,
    Decrease,
}

pub fn finalize_target(
    buffer: &Buffer,
    operator: Operator,
    target: Target,
) -> ResolveResult<Target> {
    if operator != Operator::Delete
        || target.kind != MotionKind::Char
        || target.end.line == target.start.line
    {
        return Ok(target);
    }

    let start_chars = line_chars(buffer, target.start.line).ok_or(ResolveError::InvalidPosition)?;
    let end_chars = line_chars(buffer, target.end.line).ok_or(ResolveError::InvalidPosition)?;

    let head_blank = start_chars
        .iter()
        .take(target.start.col)
        .all(|c| is_blank(*c));
    let tail_blank = end_chars.iter().skip(target.end.col).all(|c| is_blank(*c));

    if head_blank && tail_blank {
        Ok(Target::lines(target.start.line, target.end.line))
    } else {
        Ok(target)
    }
}
