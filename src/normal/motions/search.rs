use crate::{
    editor::buffer::Buffer,
    normal::{
        motion::{MotionState, SearchSpec},
        resolver::{ResolveError, ResolveResult},
        scanner::{is_keyword, line_chars},
        target::{Inclusivity, MotionOutput, Position},
    },
};

fn matches_at(chars: &[char], pattern: &[char], col: usize, whole_word: bool) -> bool {
    if col + pattern.len() > chars.len() || chars[col..col + pattern.len()] != *pattern {
        return false;
    }

    if !whole_word {
        return true;
    }

    let before = col.checked_sub(1).and_then(|i| chars.get(i)).copied();
    let after = chars.get(col + pattern.len()).copied();

    !before.is_some_and(is_keyword) && !after.is_some_and(is_keyword)
}

fn line_matches(chars: &[char], pattern: &[char], whole_word: bool) -> Vec<usize> {
    if pattern.is_empty() {
        return Vec::new();
    }

    (0..chars.len())
        .filter(|&col| matches_at(chars, pattern, col, whole_word))
        .collect()
}

fn find_once(
    buffer: &Buffer,
    from: Position,
    pattern: &[char],
    forward: bool,
    whole_word: bool,
) -> Option<Position> {
    let total = buffer.line_count();

    if total == 0 || pattern.is_empty() {
        return None;
    }

    for step in 0..=total {
        let line = if forward {
            (from.line + step) % total
        } else {
            (from.line + total - step % total) % total
        };

        let chars = line_chars(buffer, line)?;
        let hits = line_matches(&chars, pattern, whole_word);

        let found = if forward {
            if step == 0 {
                hits.into_iter().find(|&col| col > from.col)
            } else if step == total {
                hits.into_iter().find(|&col| col <= from.col)
            } else {
                hits.into_iter().next()
            }
        } else if step == 0 {
            hits.into_iter().rev().find(|&col| col < from.col)
        } else if step == total {
            hits.into_iter().rev().find(|&col| col >= from.col)
        } else {
            hits.into_iter().next_back()
        };

        if let Some(col) = found {
            return Some(Position::new(line, col));
        }
    }

    None
}

fn search(
    buffer: &Buffer,
    position: Position,
    pattern: &str,
    forward: bool,
    count: usize,
    whole_word: bool,
) -> ResolveResult<MotionOutput> {
    let pattern: Vec<char> = pattern.chars().collect();
    let mut current = position;

    for _ in 0..count {
        current = find_once(buffer, current, &pattern, forward, whole_word)
            .ok_or(ResolveError::Failed)?;
    }

    Ok(MotionOutput::chars(current, Inclusivity::Exclusive))
}

pub fn word_under_cursor(buffer: &Buffer, position: Position) -> ResolveResult<String> {
    let chars = line_chars(buffer, position.line).ok_or(ResolveError::InvalidPosition)?;

    let start = (position.col..chars.len())
        .find(|&col| is_keyword(chars[col]))
        .ok_or(ResolveError::Failed)?;

    let begin = (0..=start)
        .rev()
        .take_while(|&col| is_keyword(chars[col]))
        .last()
        .unwrap_or(start);

    let end = (start..chars.len())
        .take_while(|&col| is_keyword(chars[col]))
        .last()
        .unwrap_or(start);

    Ok(chars[begin..=end].iter().collect())
}

pub fn start(
    buffer: &Buffer,
    position: Position,
    pattern: &str,
    forward: bool,
    count: usize,
    state: &mut MotionState,
) -> ResolveResult<MotionOutput> {
    if pattern.is_empty() {
        return Err(ResolveError::Failed);
    }

    state.last_search = Some(SearchSpec {
        pattern: pattern.to_string(),
        forward,
    });

    search(buffer, position, pattern, forward, count, false)
}

pub fn repeat(
    buffer: &Buffer,
    position: Position,
    count: usize,
    reverse: bool,
    state: &mut MotionState,
) -> ResolveResult<MotionOutput> {
    let spec = state.last_search.clone().ok_or(ResolveError::Failed)?;
    let forward = spec.forward != reverse;

    search(buffer, position, &spec.pattern, forward, count, false)
}

pub fn word(
    buffer: &Buffer,
    position: Position,
    count: usize,
    forward: bool,
    state: &mut MotionState,
) -> ResolveResult<MotionOutput> {
    let word = word_under_cursor(buffer, position)?;

    let chars = line_chars(buffer, position.line).ok_or(ResolveError::InvalidPosition)?;
    let word_start = (0..=position.col.min(chars.len().saturating_sub(1)))
        .rev()
        .take_while(|&col| is_keyword(chars[col]))
        .last()
        .unwrap_or(position.col);
    let origin = Position::new(position.line, word_start.min(position.col));

    state.last_search = Some(SearchSpec {
        pattern: word.clone(),
        forward,
    });

    search(buffer, origin, &word, forward, count, true)
}
