use std::time::Instant;

pub enum MessageLevel {
    Success,
    Error,
}

pub struct Message {
    pub message: String,
    pub level: MessageLevel,
    pub duration: Instant,
}
