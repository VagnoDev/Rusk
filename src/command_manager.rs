use std::io::Result;
use std::io::stdout;

use crossterm::terminal;
use crossterm:: execute;

use crate::Status::Continue;
use crate::Status::Quit;
use crate::save::save;
use crate::Buffer;
use crate::Status;

pub fn execute(command: &str, buffer: &Buffer, path: &str) -> std::io::Result<Status> {
    let mut parts = command.split_whitespace();

    let Some(name) = parts.next() else {
        return Ok(Quit);
    };
    let _argument = parts.next();

    match name {
        "q" => Ok(Quit),
        "w" => { 
            save(buffer, &path.to_string())?;
            Ok(Continue)
        },
        "wq" => {
            save(buffer, &path.to_string())?;
            terminal::disable_raw_mode()?;
            execute!(stdout(), terminal::LeaveAlternateScreen)?;
            Ok(Quit)
        }
        _ => Ok(Continue)
    }
}