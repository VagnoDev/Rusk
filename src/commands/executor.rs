use std::io::stdout;

use crossterm::{
    execute,
    terminal::{self, LeaveAlternateScreen},
};

use crate::io::save::save;
use crate::{Buffer, Status};

use super::parser::Command;

pub fn execute(
    command: Command,
    buffer: &Buffer,
    path: &str,
) -> std::io::Result<Status> {
    match command {
        Command::Quit => Ok(Status::Quit),

        Command::Write => {
            save(buffer, &path.to_string())?;
            Ok(Status::Continue)
        }

        Command::WriteQuit => {
            save(buffer, &path.to_string())?;
            terminal::disable_raw_mode()?;
            execute!(stdout(), LeaveAlternateScreen)?;
            Ok(Status::Quit)
        }

        Command::Unknown => Ok(Status::Continue),
    }
}