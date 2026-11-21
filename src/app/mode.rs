use crate::app::context::EditorContext;

#[derive(Debug)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Explorer,
}

pub fn set_normal(context: &mut EditorContext) {
    context.mode = Mode::Normal;

    context.command_buffer.clear();
    context.search_forward = None;
}

pub fn set_command(context: &mut EditorContext) {
    context.search_line = None;
    context.mode = Mode::Command;
}

pub fn set_search(context: &mut EditorContext, forward: bool) {
    context.mode = Mode::Command;
    context.search_line = None;
    context.command_buffer.clear();
    context.search_forward = Some(forward);
}
