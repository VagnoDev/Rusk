use std::{char, io, task::Context};

use crossterm::{
    event::{self, Event, KeyEventKind},
};
use crate::gui::display_manager::{Cell, Display};

use crate::{
    buffer::{insert_char, insert_forward, move_down, move_left, move_right, move_up, set_normal}, input::{
        Action::{self},
        keybinding,
    },
};
use crossterm::terminal;
use std::collections::VecDeque;

use crate::buffer::Buffer;
use crate::context_manager::EditorContext;
use crate::input::Mode;

mod buffer;
mod context_manager;
mod gui;
mod input;
mod save;

fn main() -> std::io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut lines = buffer::fetch("src/test.txt");

    let mut display = Display::new()?;
    display.draw(&mut lines)?;
    //println!("{:#?}", lines);

    let mut context = EditorContext { mode: Mode::Normal };

    let mut queue = VecDeque::new();

    loop {
        match event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    if let Some(realbind) = keybinding(key, &context.mode) {
                        queue.push_back(realbind);
                    }
                }
            }
            _ => {}
        }
        manager(&mut queue, &mut lines, &mut context);
        display.draw(&mut lines)?;
    }
}

fn manager(queue: &mut VecDeque<Action>, lines: &mut Buffer, context: &mut EditorContext) {
    let Some(first) = queue.pop_front() else {
        return;
    };
    match context.mode {
        Mode::Normal => match first {
            Action::MoveLeft => {
                move_left(lines)
                
            },
            Action::MoveRight => move_right(lines),
            Action::MoveUp => move_up(lines),
            Action::MoveDown => move_down(lines),
            Action::InsertForward => insert_forward(lines, context),
            _ => {}
        },
        Mode::Insert => match first {
            Action::InsertChar(char) => insert_char(char, lines),
            Action::SetNormal => set_normal(context),
            _ => {}
        },
        Mode::Command => {
            return;
        }
    }

    //println!("{:?}", lines);
}
