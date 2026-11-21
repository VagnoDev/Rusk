use crate::{
    editor::buffer::Buffer,
    normal::{
        motions::line::first_non_blank_on,
        resolver::{ResolveError, ResolveResult},
        target::MotionOutput,
    },
};

fn last_line(buffer: &Buffer) -> ResolveResult<usize> {
    buffer
        .line_count()
        .checked_sub(1)
        .ok_or(ResolveError::InvalidPosition)
}

pub fn goto(
    buffer: &Buffer,
    count: Option<usize>,
    default_end: bool,
) -> ResolveResult<MotionOutput> {
    let last = last_line(buffer)?;

    let line = match count {
        Some(n) => last.min(n - 1),
        None if default_end => last,
        None => 0,
    };

    first_non_blank_on(buffer, line)
}

pub fn percent(buffer: &Buffer, n: usize) -> ResolveResult<MotionOutput> {
    if n == 0 || n > 100 {
        return Err(ResolveError::Failed);
    }

    let total = last_line(buffer)? + 1;
    let line = ((total * n).div_ceil(100)).saturating_sub(1).min(total - 1);

    first_non_blank_on(buffer, line)
}
