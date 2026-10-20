use crate::input::Mode;

pub struct EditorContext {
    pub mode: Mode,
    pub command_buffer: String,
    pub file_path: String
}