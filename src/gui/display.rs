use crate::{
    OpenBuffer,
    app::{context::EditorContext, mode::Mode},
    editor::buffer::Buffer,
    explorer::tree::Explorer,
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal,
};

use std::io::{BufWriter, Write, stdout};

use super::{
    editor_view::{self, gutter_width},
    explorer_view, status_bar,
};

#[derive(Default, Clone, Copy, PartialEq)]
pub(super) enum Colors {
    #[default]
    Black,
    White,
    Red,
    Reset,
    Rgb(u8, u8, u8),
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct Cell {
    pub(super) symbol: char,
    pub(super) fg: Colors,
    pub(super) bg: Colors,
}

pub struct Display {
    pub(super) cells: Vec<Cell>,
    previous: Vec<Cell>,
    pub(super) width: u32,
    pub height: u32,
}

pub(super) fn terminal_to_color(color: &Colors) -> Color {
    match color {
        Colors::Black => Color::Black,
        Colors::White => Color::White,
        Colors::Red => Color::Red,
        Colors::Reset => Color::Reset,
        Colors::Rgb(r, g, b) => Color::Rgb {
            r: *r,
            g: *g,
            b: *b,
        },
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            symbol: ' ',
            fg: Colors::White,
            bg: Colors::Reset,
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
            previous: Vec::new(),
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
        buffers: &[OpenBuffer],
    ) -> std::io::Result<()> {
        let mut stdout = BufWriter::with_capacity(1 << 16, stdout().lock());

        queue!(stdout, Hide)?;

        for cell in &mut self.cells {
            *cell = Cell::default();
        }

        let editor_offset = if sidebar_visible { 26 } else { 0 };

        editor_view::draw_editor(self, buffer, editor_offset, context);

        if sidebar_visible {
            explorer_view::draw_explorer(self, explorer, buffers);
        }

        status_bar::draw_status_bar(self, context, buffer);

        let width = self.width as usize;

        let full = self.previous.len() != self.cells.len();
        let mut next_index = None;
        let mut colors = None;

        for (i, cell) in self.cells.iter().enumerate() {
            if width == 0 {
                break;
            }

            if !full && self.previous[i] == *cell {
                continue;
            }

            if next_index != Some(i) || i % width == 0 {
                queue!(stdout, MoveTo((i % width) as u16, (i / width) as u16))?;
            }

            if colors != Some((cell.fg, cell.bg)) {
                queue!(
                    stdout,
                    SetForegroundColor(terminal_to_color(&cell.fg)),
                    SetBackgroundColor(terminal_to_color(&cell.bg))
                )?;
                colors = Some((cell.fg, cell.bg));
            }

            queue!(stdout, Print(cell.symbol))?;
            next_index = cell.symbol.is_ascii().then_some(i + 1);
        }

        self.previous.clone_from(&self.cells);

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
                    MoveTo(
                        (editor_offset + gutter_width(buffer) + buffer.row) as u16,
                        buffer.column.saturating_sub(context.viewport_top) as u16,
                    ),
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
        self.previous.clear();
    }
}
