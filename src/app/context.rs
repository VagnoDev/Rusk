use std::path::PathBuf;

use crate::app::messages::Message;
use crate::app::mode::Mode;

pub struct EditorContext {
    pub mode: Mode,
    pub command_buffer: String,
    pub file_path: Option<PathBuf>,
    pub current_message: Option<Message>,
    pub savedialog: bool,
    pub quit_after_save: bool
}
