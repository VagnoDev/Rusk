use std::io::stdout;

use crossterm::execute;

use crate::app::context::EditorContext;
use crate::io::save::save;
use crate::normal::resolver::ResolveError;
use crate::report;
use crate::{Buffer, Status};

use super::parser::Command;

pub fn execute(
    command: Command,
    buffer: &mut Buffer,
    context: &mut EditorContext,
) -> std::io::Result<Status> {
    match command {
        Command::Quit => {
            if buffer.modified {
                report(
                    context,
                    ResolveError::Failed,
                    "No write since last change (add ! to override)",
                );

                return Ok(Status::Continue);
            }
            Ok(Status::Quit)
        }

        Command::ForceQuit => Ok(Status::Quit),

        Command::Write(command_path) => {
            match command_path {
                Some(new_path) => save(buffer, std::path::Path::new(&new_path))?,
                None => {
                    if let Some(path) = context.file_path.as_ref() {
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
                    if let Some(path) = context.file_path.as_ref() {
                        save(buffer, path)?;
                    }
                }
            }

            Ok(Status::Quit)
        }

        Command::Unknown => Ok(Status::Continue),
    }
}
