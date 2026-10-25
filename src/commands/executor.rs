use std::{
    io::stdout,
    path::{self, Path, PathBuf},
};

use crossterm::{
    execute,
    terminal::{self, LeaveAlternateScreen},
};

use crate::{Buffer, Status};
use crate::{
    app::{
        context::{self, EditorContext},
        messages::MessageLevel,
    },
    gui::status_bar::send_message,
    io::save::save,
};

use super::parser::Command;

pub fn execute(
    command: Command,
    buffer: &Buffer,
    path: &Option<PathBuf>,
) -> std::io::Result<Status> {
    match command {
        Command::Quit => Ok(Status::Quit),

        Command::Write(command_path) => {
            match command_path {
                Some(new_path) => save(buffer, std::path::Path::new(&new_path))?,
                None => {
                    if let Some(path) = path {
                        save(buffer, path)?;
                    }
                }
            }
            execute!(stdout(), crossterm::cursor::Hide)?;
            Ok(Status::Continue)
        }

        Command::WriteQuit(command_path) => {
            match command_path {
                Some(new_path) => save(buffer, std::path::Path::new(&new_path))?,
                None => {
                    if let Some(path) = path {
                        save(buffer, path)?;
                    }
                }
            }
        
            Ok(Status::Quit)
        }

        Command::Unknown => Ok(Status::Continue),
    }
}
