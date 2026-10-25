use crate::{
    app::context::EditorContext, app::mode::Mode, editor::buffer::Buffer, explorer::tree::Explorer,
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal,
};

use std::io::{Write, stdout};

use super::{editor_view, explorer_view, status_bar};

#[derive(Default)]
pub(super) enum Colors {
    #[default]
    BLACK,
    WHITE,
    RED,
    GREEN,
    BLUE,
    CYAN,
}

pub(super) struct Cell {
    pub(super) symbol: char,
    pub(super) fg: Colors,
    pub(super) bg: Colors,
}

pub struct Display {
    pub(super) cells: Vec<Cell>,
    pub(super) width: u32,
    pub(super) height: u32,
}

pub(super) fn terminal_to_color(color: &Colors) -> Color {
    match color {
        Colors::BLACK => Color::Black,
        Colors::WHITE => Color::White,
        Colors::RED => Color::Red,
        Colors::GREEN => Color::Green,
        Colors::BLUE => Color::Blue,
        Colors::CYAN => Color::Cyan,
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            symbol: ' ',
            fg: Colors::WHITE,
            bg: Colors::BLACK,
        }
    }
}

impl Display {
    pub fn new() -> std::io::Result<Self> {
        let (width, height) = terminal::size()?;
        let space_to_allocate = width as usize * height as usize;

        let mut cells = Vec::new();
        cells.resize_with(space_to_allocate, Cell::default);

        Ok(Self {
            cells,
            width: width as u32,
            height: height as u32,
        })
    }

    pub fn draw(
        &mut self,
        buffer: &Buffer,
        context: &EditorContext,
        explorer: &Explorer,
        sidebar_visible: bool,
    ) -> std::io::Result<()> {
        let mut stdout = stdout();

        queue!(stdout, Hide)?;

        for cell in &mut self.cells {
            *cell = Cell::default();
        }

        let editor_offset = if sidebar_visible { 26 } else { 0 };

        editor_view::draw_editor(self, buffer, editor_offset);

        if sidebar_visible {
            explorer_view::draw_explorer(self, explorer);
        }

        status_bar::draw_status_bar(self, context);

        let width = self.width as usize;

        for (i, cell) in self.cells.iter().enumerate() {
            if width == 0 {
                break;
            }

            let x = i % width;
            let y = i / width;

            queue!(stdout, MoveTo(x as u16, y as u16))?;
            queue!(
                stdout,
                SetForegroundColor(terminal_to_color(&cell.fg)),
                SetBackgroundColor(terminal_to_color(&cell.bg)),
                Print(cell.symbol)
            )?;
        }

        match context.mode {
            Mode::Command => {
                let x = if context.savedialog {
                    (context.command_buffer.chars().count() + 9) as u16
                } else {
                    (context.command_buffer.chars().count() + 1) as u16
                };
                queue!(
                    stdout,
                    MoveTo(x, self.height.saturating_sub(1) as u16,),
                    Show
                )?;
            }
            Mode::Explorer => {
                execute!(stdout, Hide)?;
            }
            _ => {
                queue!(
                    stdout,
                    MoveTo(buffer.row as u16, buffer.column as u16),
                    Show
                )?;
            }
        }
        stdout.flush()?;

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        let space_to_allocate = width as usize * height as usize;

        let mut cells = Vec::new();
        cells.resize_with(space_to_allocate, Cell::default);

        self.cells = cells;
    }
}
