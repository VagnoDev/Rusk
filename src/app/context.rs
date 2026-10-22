use crate::app::messages::Message;
use crate::app::mode::Mode;

pub struct EditorContext {
    pub mode: Mode,
    pub command_buffer: String,
    pub file_path: String,
    pub current_message: Option<Message>,
}
