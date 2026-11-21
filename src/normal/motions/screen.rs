use crate::{
    editor::buffer::Buffer,
    normal::{
        motion::Viewport,
        motions::line::first_non_blank_on,
        resolver::{ResolveError, ResolveResult},
        target::MotionOutput,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenPos {
    Top,
    Middle,
    Bottom,
}

pub fn resolve(
    buffer: &Buffer,
    pos: ScreenPos,
    count: usize,
    viewport: Viewport,
) -> ResolveResult<MotionOutput> {
    let last = buffer
        .line_count()
        .checked_sub(1)
        .ok_or(ResolveError::InvalidPosition)?;

    if viewport.height == 0 {
        return Err(ResolveError::Failed);
    }

    let top = viewport.top.min(last);
    let bottom = (viewport.top + viewport.height - 1).min(last);
    let offset = count - 1;

    let line = match pos {
        ScreenPos::Top => (top + offset).min(bottom),
        ScreenPos::Middle => top + (bottom - top) / 2,
        ScreenPos::Bottom => bottom.saturating_sub(offset).max(top),
    };

    first_non_blank_on(buffer, line)
}
