use std::io::stdout;
use std::path::Path;

use crossterm::execute;

use crate::app::context::EditorContext;
use crate::io::save::save;
use crate::normal::resolver::ResolveError;
use crate::{Buffer, Status};
use crate::{report, same_file};

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
                Some(new_path) => save_as(buffer, context, &new_path)?,
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
                Some(new_path) => save_as(buffer, context, &new_path)?,
                None => {
                    if let Some(path) = context.file_path.as_ref() {
                        save(buffer, path)?;
                    }
                }
            }

            Ok(Status::Quit)
        }

        Command::Edit(_) | Command::ForceEdit(_) | Command::Unknown => Ok(Status::Continue),
    }
}

fn save_as(
    buffer: &mut Buffer,
    context: &mut EditorContext,
    new_path: &str,
) -> std::io::Result<()> {
    let path = Path::new(new_path);
    let modified = buffer.modified;
    save(buffer, path)?;

    match context.file_path.as_ref() {
        None => context.file_path = Some(path.to_path_buf()),
        Some(current) if !same_file(current, path) => buffer.modified = modified,
        Some(_) => {}
    }

    Ok(())
}
