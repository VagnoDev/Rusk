use crate::{
    editor::buffer::Buffer,
    normal::{
        resolver::{ResolveError, ResolveResult},
        scanner::{is_blank, line_chars},
        target::{Inclusivity, MotionOutput, Position},
    },
};

fn starts(buffer: &Buffer) -> Vec<Position> {
    let mut found = Vec::new();
    let mut need_start = true;

    for line in 0..buffer.line_count() {
        let chars = line_chars(buffer, line).unwrap_or_default();

        if chars.is_empty() {
            found.push(Position::new(line, 0));
            need_start = true;
            continue;
        }

        let mut col = 0;

        while col < chars.len() {
            let c = chars[col];

            if need_start && !is_blank(c) {
                found.push(Position::new(line, col));
                need_start = false;
            }

            if matches!(c, '.' | '!' | '?') {
                let mut end = col + 1;

                while end < chars.len() && matches!(chars[end], ')' | ']' | '"' | '\'') {
                    end += 1;
                }

                if end >= chars.len() || is_blank(chars[end]) {
                    need_start = true;
                    col = end;
                    continue;
                }
            }

            col += 1;
        }
    }

    found
}

pub fn resolve(
    buffer: &Buffer,
    position: Position,
    count: usize,
    forward: bool,
) -> ResolveResult<MotionOutput> {
    let all = starts(buffer);

    let target = if forward {
        all.iter().copied().filter(|p| *p > position).nth(count - 1)
    } else {
        all.iter()
            .rev()
            .copied()
            .filter(|p| *p < position)
            .nth(count - 1)
    };

    if let Some(pos) = target {
        return Ok(MotionOutput::chars(pos, Inclusivity::Exclusive));
    }

    if forward {
        let last = buffer.line_count() - 1;
        let len = line_chars(buffer, last).map_or(0, |c| c.len());
        let end = Position::new(last, len.saturating_sub(1));

        if end == position {
            return Err(ResolveError::Failed);
        }

        return Ok(MotionOutput::chars(end, Inclusivity::Inclusive));
    }

    let top = Position::new(0, 0);

    if top == position {
        return Err(ResolveError::Failed);
    }

    Ok(MotionOutput::chars(top, Inclusivity::Exclusive))
}
