use crate::app::mode::Mode;

pub struct EditorContext {
    pub mode: Mode,
    pub command_buffer: String,
    pub file_path: String
}