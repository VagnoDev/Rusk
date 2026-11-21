use crate::{
    editor::buffer::Buffer,
    normal::{
        motion::MotionState,
        motions::line::first_non_blank_col,
        resolver::{ResolveError, ResolveResult},
        scanner::{line_chars, line_len},
        target::{Inclusivity, MotionOutput, Position},
    },
};

pub const JUMP_MARK: char = '\'';

fn is_valid_name(name: char) -> bool {
    name.is_ascii_alphabetic() || name == JUMP_MARK || name == '`'
}

fn canonical(name: char) -> char {
    if name == '`' { JUMP_MARK } else { name }
}

pub fn set(state: &mut MotionState, name: char, position: Position) -> ResolveResult<()> {
    if !is_valid_name(name) {
        return Err(ResolveError::Failed);
    }

    state.marks.insert(canonical(name), position);

    Ok(())
}

pub fn record_jump(state: &mut MotionState, position: Position) {
    state.marks.insert(JUMP_MARK, position);
}

pub fn resolve(
    buffer: &Buffer,
    name: char,
    linewise: bool,
    state: &MotionState,
) -> ResolveResult<MotionOutput> {
    if !is_valid_name(name) {
        return Err(ResolveError::Failed);
    }

    let mark = *state
        .marks
        .get(&canonical(name))
        .ok_or(ResolveError::Failed)?;

    let last = buffer
        .line_count()
        .checked_sub(1)
        .ok_or(ResolveError::InvalidPosition)?;
    let line = mark.line.min(last);

    if linewise {
        let chars = line_chars(buffer, line).ok_or(ResolveError::InvalidPosition)?;

        return Ok(MotionOutput::lines(Position::new(
            line,
            first_non_blank_col(&chars),
        )));
    }

    let len = line_len(buffer, line).ok_or(ResolveError::InvalidPosition)?;
    let col = mark.col.min(len.saturating_sub(1));

    Ok(MotionOutput::chars(
        Position::new(line, col),
        Inclusivity::Exclusive,
    ))
}
