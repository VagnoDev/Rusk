use crate::app::context::EditorContext;

#[derive(Debug)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Explorer
}

pub fn set_normal(context: &mut EditorContext) {
    context.mode = Mode::Normal;

    context.command_buffer.clear();
}

pub fn set_command(context: &mut EditorContext) {
    context.mode = Mode::Command;
}
