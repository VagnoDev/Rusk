use std::path::PathBuf;

use crate::app::messages::Message;
use crate::app::mode::Mode;
use crate::normal::apply::Register;
use crate::normal::motion::MotionState;
use crate::normal::parser::Parser;

pub struct EditorContext {
    pub mode: Mode,
    pub command_buffer: String,
    pub file_path: Option<PathBuf>,
    pub current_message: Option<Message>,
    pub savedialog: bool,
    pub quit_after_save: bool,
    pub viewport_top: usize,
    pub search_forward: Option<bool>,
    pub motion_state: MotionState,
    pub parser: Parser,
    pub pending_space: bool,
    pub register: Option<Register>,
    pub search_line: Option<String>,
    pub pending_keys: String,
}
