use std::{fs::read_dir, io::stdout, path::PathBuf};

use crate::{
    Status::Continue, buffer::{insert_newline, remove_char, set_command}, command_manager::execute, dir_manager::Explorer, gui::display_manager::Display, input::{Action::Quit, PendingKey},
};
use crossterm::{
    event::{self, Event::{self}, KeyEventKind}, execute,
};

use std::path::Path;

use crate::{
    buffer::{insert_char, insert_forward, move_down, move_left, move_right, move_up, set_normal},
    input::{
        Action::{self},
    },
};
use crossterm::terminal;
use std::collections::VecDeque;

use crate::buffer::Buffer;
use crate::context_manager::EditorContext;
use crate::input::Mode;
use crate::input::keybinding;

mod buffer;
mod command_manager;
mod context_manager;
mod gui;
mod input;
mod save;
mod dir_manager;

pub struct App {
    editor: buffer::Buffer,
    explorer: dir_manager::Explorer,
    sidebar_visible: bool
}

pub enum Status {
    Continue,
    Quit,
    OpenExplorer,
    CloseExplorer
}

fn main() -> std::io::Result<()> {
    terminal::enable_raw_mode()?;
    execute!(
        stdout(),
        terminal::EnterAlternateScreen,
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        crossterm::cursor::MoveTo(0, 0)
    )?;
    let path = std::env::args()
        .nth(1)
        .expect("Use: rusk <nomefile>");

    let mut app = match Path::new(&path).is_dir() {
        true => App {
            editor: Buffer {
                lines: vec![String::new()],
                row: 0,
                column: 0,
            },
            explorer: dir_manager::read_dir(&PathBuf::from(&path), 0),
            sidebar_visible: false,
        },
        false => {
            let file_path = Path::new(&path);
    
            let parent = file_path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
    
            App {
                editor: buffer::fetch(&path),
                explorer: dir_manager::read_dir(&PathBuf::from(parent), 0),
                sidebar_visible: false,
            }
        }
    };

    let mut context = EditorContext {
        mode: Mode::Normal,
        command_buffer: String::new(),
        file_path: path.clone(),
    };

    let mut display = Display::new()?;

    let mut queue = VecDeque::new();
    let mut pending_key = PendingKey::None;

    display.draw(&mut app.editor, &context, &app.explorer, app.sidebar_visible)?;

    loop {
        match event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    if let Some(realbind) = keybinding(key, &context.mode, &mut pending_key) {
                        queue.push_back(realbind);
                    }
                }
            }
            Event::Resize(width, height) => {
                display.resize(width as u32, height as u32);
            }
            _ => {}
        }
        let status = manager(&mut queue, &mut app.editor, &mut context, &mut app.explorer)?;

        match status {
            Status::Continue => {},
            Status::OpenExplorer => {
                app.sidebar_visible = true;
                context.mode = Mode::Explorer
            }
            Status::CloseExplorer =>{
                app.sidebar_visible = false;
                context.mode = Mode::Normal
            }
            Status::Quit => break,
        }
        
    }
    terminal::disable_raw_mode()?;
    execute!(stdout(), terminal::LeaveAlternateScreen)?;
    Ok(())
}

fn manager(
    queue: &mut VecDeque<Action>,
    lines: &mut Buffer,
    context: &mut EditorContext,
    explorer: &mut Explorer
) -> std::io::Result<Status> {
    let Some(first) = queue.pop_front() else {
        return Ok(Continue);
    };
    match context.mode {
        Mode::Normal => match first {
            Action::MoveLeft => move_left(lines),
            Action::MoveRight => move_right(lines),
            Action::MoveUp => move_up(lines),
            Action::MoveDown => move_down(lines),
            Action::InsertForward => insert_forward(lines, context),
            Action::SetCommand => set_command(context),
            Action::SetExplorer => {
                return Ok(Status::OpenExplorer);
            },
            _ => {}
        },
        Mode::Insert => match first {
            Action::InsertChar(char) => insert_char(char, lines),
            Action::SetNormal => set_normal(context),
            Action::RemoveChar => remove_char(lines),
            Action::InserNewLine => insert_newline(lines),
            _ => {}
        },
        Mode::Command => match first {
            Action::EscCommand => set_normal(context),
            Action::InsertChar(char) => context.command_buffer.push(char),
            Action::RemoveChar =>  { 
                context.command_buffer.pop();
            },
            Action::SendCommand => {
                return execute(context.command_buffer.as_str(), lines, &context.file_path);
            },
            _ => {}
        },
        Mode::Explorer => match first {
            Action::SetNormal => {
                set_normal(context);
                return Ok(Status::CloseExplorer)
            },
            Action::ExplorerDown => explorer.explorer_down(),
            Action::ExplorerUp => explorer.explorer_up(),
            Action::EnterFile => {
                if explorer.entries[explorer.selected].is_dir{
                    explorer.explorer_toggle()
                } else {
                   let path = explorer.entries[explorer.selected].path.clone();
                   if let Some(path) = path.to_str() {
                       *lines = buffer::fetch(path);
                       context.file_path = path.to_string();
                   }
                }
            } 
            _ => {}
        }
    }
    Ok(Continue)
}
