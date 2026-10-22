use std::{io::stdout, path::PathBuf, time::Instant};

use crate::{
    Status::Continue,
    app::mode::{Mode, set_command, set_normal},
    commands::{executor, parser},
    editor::editing::{insert_newline, remove_char},
    editor::movement::{
        move_down, move_left, move_right, move_toend, move_tonext, move_toprev, move_tostart,
        move_up,
    },
    explorer::tree::{Explorer, read_dir},
    gui::display::Display,
    input::keyboard::{PendingKey, keybinding},
    gui::status_bar::send_message,
    app::messages::MessageLevel
};
use crossterm::{
    cursor, event::{
        self,
        Event::{self},
        KeyEventKind,
    }, execute,
};

use std::path::Path;

use crate::{
    app::actions::Action::{self},
    editor::buffer,
    editor::editing::{insert_char, insert_forward},
};
use crossterm::terminal;
use std::collections::VecDeque;

use crate::app::context;
use crate::context::EditorContext;
use crate::editor::buffer::Buffer;

mod app;
mod commands;
mod editor;
mod explorer;
mod gui;
mod input;
mod io;

pub struct App {
    editor: buffer::Buffer,
    explorer: Explorer,
    sidebar_visible: bool,
}

pub enum Status {
    Continue,
    Quit,
    OpenExplorer,
    CloseExplorer,
}

fn main() -> std::io::Result<()> {
    terminal::enable_raw_mode()?;
    execute!(
        stdout(),
        terminal::EnterAlternateScreen,
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        crossterm::cursor::MoveTo(0, 0)
    )?;
    let path = std::env::args().nth(1).expect("Use: rusk <nomefile>");

    let mut app = match Path::new(&path).is_dir() {
        true => App {
            editor: Buffer {
                lines: vec![String::new()],
                row: 0,
                column: 0,
            },
            explorer: read_dir(&PathBuf::from(&path), 0),
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
                explorer: read_dir(&PathBuf::from(parent), 0),
                sidebar_visible: false,
            }
        }
    };

    let mut context = EditorContext {
        mode: Mode::Normal,
        command_buffer: String::new(),
        file_path: path.clone(),
        current_message: None,
    };

    let mut display = Display::new()?;

    let mut queue = VecDeque::new();
    let mut pending_key = PendingKey::None;

    display.draw(
        &mut app.editor,
        &context,
        &app.explorer,
        app.sidebar_visible,
    )?;

    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
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
            let expired = match &context.current_message {
                Some(message) => Instant::now() >= message.duration,
                None => false,
            };
            if expired {
                context.current_message = None;
            }
        }
        let status = manager(&mut queue, &mut app.editor, &mut context, &mut app.explorer)?;

        match status {
            Status::Continue => {}
            Status::OpenExplorer => {
                app.sidebar_visible = true;
                context.mode = Mode::Explorer
            }
            Status::CloseExplorer => {
                app.sidebar_visible = false;
                context.mode = Mode::Normal
            }
            Status::Quit => break,
        }

        display.draw(
            &mut app.editor,
            &context,
            &app.explorer,
            app.sidebar_visible,
        )?;
    }
    terminal::disable_raw_mode()?;
    execute!(stdout(), terminal::LeaveAlternateScreen)?;
    Ok(())
}

fn manager(
    queue: &mut VecDeque<Action>,
    lines: &mut Buffer,
    context: &mut EditorContext,
    explorer: &mut Explorer,
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
            Action::StartLine => move_tostart(lines),
            Action::EndLine => move_toend(lines),
            Action::MoveNext => move_tonext(lines),
            Action::MovePrev => move_toprev(lines),
            Action::SetExplorer => {
                return Ok(Status::OpenExplorer);
            }
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
            Action::RemoveChar => {
                context.command_buffer.pop();
            }
            Action::SendCommand => {
                let command = parser::parse(context.command_buffer.as_str());
            
                let is_write = matches!(&command, parser::Command::Write);
            
                let status = executor::execute(
                    command,
                    lines,
                    &context.file_path,
                )?;
            
                if is_write {
                    set_normal(context);
                    send_message(
                        context,
                        "File Saved.".to_string(),
                        MessageLevel::Success,
                    );
                }
            
                return Ok(status);
            }
            _ => {}
        },
        Mode::Explorer => match first {
            Action::SetNormal => {
                set_normal(context);
                return Ok(Status::CloseExplorer);
            }
            Action::ExplorerDown => explorer.explorer_down(),
            Action::ExplorerUp => explorer.explorer_up(),
            Action::EnterFile => {
                if explorer.entries[explorer.selected].is_dir {
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
        },
    }
    Ok(Continue)
}
