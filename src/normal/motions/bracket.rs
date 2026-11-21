use crate::{
    editor::buffer::Buffer,
    normal::{
        resolver::{ResolveError, ResolveResult},
        scanner::{Cursor, Step, line_chars},
        target::{Inclusivity, MotionOutput, Position},
    },
};

fn pair(character: char) -> Option<(char, bool)> {
    match character {
        '(' => Some((')', true)),
        ')' => Some(('(', false)),
        '[' => Some((']', true)),
        ']' => Some(('[', false)),
        '{' => Some(('}', true)),
        '}' => Some(('{', false)),
        _ => None,
    }
}

pub fn resolve(buffer: &Buffer, position: Position) -> ResolveResult<MotionOutput> {
    let chars = line_chars(buffer, position.line).ok_or(ResolveError::InvalidPosition)?;

    let col = (position.col..chars.len())
        .find(|&col| pair(chars[col]).is_some())
        .ok_or(ResolveError::Failed)?;

    let origin = chars[col];
    let (target, forward) = pair(origin).ok_or(ResolveError::Failed)?;

    let mut cursor = Cursor::new(buffer, Position::new(position.line, col))?;
    let mut depth = 0usize;

    loop {
        let step = if forward { cursor.inc() } else { cursor.dec() };

        if step == Step::Stuck {
            return Err(ResolveError::Failed);
        }

        match cursor.current() {
            Some(character) if character == target => {
                if depth == 0 {
                    return Ok(MotionOutput::chars(cursor.pos(), Inclusivity::Inclusive));
                }

                depth -= 1;
            }
            Some(character) if character == origin => depth += 1,
            _ => {}
        }
    }
}
