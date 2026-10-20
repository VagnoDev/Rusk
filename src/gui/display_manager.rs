use crate::{Mode, buffer::Buffer, context_manager::EditorContext, dir_manager::Explorer};
use crossterm::terminal;
use crossterm::{
    cursor::MoveTo,
    execute, queue,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
};
use std::{
    char,
    io::{Write, stdout},
};

#[derive(Default)]
enum Colors {
    #[default]
    BLACK,
    WHITE,
    RED,
    GREEN,
    BLUE,
    CYAN,
}

pub struct Cell {
    symbol: char,
    fg: Colors,
    bg: Colors,
}

pub struct Display {
    cells: Vec<Cell>,
    width: u32,
    height: u32,
}

fn terminal_to_color(color: &Colors) -> crossterm::style::Color {
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
        let mut new_vector: Vec<Cell> = Vec::new();
        let space_to_allocate = (width as usize) * (height as usize);
        new_vector.resize_with(space_to_allocate as usize, Cell::default);

        return Ok(Self {
            cells: (new_vector),
            width: (width as u32),
            height: (height as u32),
        });
    }

    pub fn draw(
        &mut self,
        buffer: &mut Buffer,
        context: &EditorContext,
        explorer: &Explorer,
        sidebar_visible: bool,
    ) -> std::io::Result<()> {
        let mut stdout = stdout();

        execute!(stdout, crossterm::cursor::Hide)?;

        for cell in self.cells.iter_mut() {
            *cell = Cell::default();
        }

        let editor_offset = if sidebar_visible { 26 } else { 0 };

        for (index, line) in buffer.lines.iter().enumerate() {
            if index >= (self.height - 1) as usize {
                break;
            }

            for (col, char) in line.chars().enumerate() {
                if col + editor_offset as usize >= self.width as usize {
                    break;
                }
            
                let cell_index =
                    index * self.width as usize + editor_offset as usize + col;
            
                if let Some(cell) = self.cells.get_mut(cell_index) {
                    cell.symbol = char;
                }
            }
        }

        let start = (self.height as usize - 1) * self.width as usize;
        let status = match context.mode {
            Mode::Normal => "-- NORMAL --".to_string(),
            Mode::Insert => "-- INSERT --".to_string(),
            Mode::Explorer => "-- EXPLORER --".to_string(),
            Mode::Command => format!(":{}", context.command_buffer),
        };

        for (i, char) in status.chars().enumerate() {
            if let Some(cell) = self.cells.get_mut(start + i) {
                cell.symbol = char;
            }
        }

        if sidebar_visible {
            for (y, entry) in explorer.entries.iter().enumerate() {
                for x in 0..25 {
                    let char = entry.name.chars().nth(x).unwrap_or(' ');
                    let cell_index = (y as u32 * self.width + x as u32) as usize;

                    if let Some(cell) = self.cells.get_mut(cell_index) {
                        cell.symbol = char;

                        if y == explorer.selected {
                            cell.bg = Colors::BLUE;
                        } else {
                            cell.bg = Colors::BLACK
                        }
                    }
                }
            }
        }

        if sidebar_visible {
            for y in 0..self.height - 1 {
                let x = 25;
                let cell_index = (y * self.width + x) as usize;
                if let Some(cell) = self.cells.get_mut(cell_index) {
                    cell.symbol = '|';
                    cell.fg = Colors::WHITE;
                }
            }
        }
        
        for (i, cell) in self.cells.iter().enumerate() {
            let width = self.width as usize;
            let x = i % width;
            let y = i / width;
            queue!(stdout, MoveTo(x as u16, y as u16))?;
            queue!(stdout, SetForegroundColor(terminal_to_color(&cell.fg)))?;
            queue!(stdout, SetBackgroundColor(terminal_to_color(&cell.bg)))?;
            queue!(stdout, Print(cell.symbol))?;
        }

        stdout.flush()?;
        match context.mode {
            Mode::Command => {
                execute!(
                    stdout,
                    MoveTo(
                        (context.command_buffer.chars().count() + 1) as u16,
                        self.height as u16 - 1
                    ),
                    crossterm::cursor::Show
                )
            }
            Mode::Explorer => {
                execute!(stdout, crossterm::cursor::Hide)
            }
            _ => {
                execute!(
                    stdout,
                    MoveTo(buffer.row as u16, buffer.column as u16),
                    crossterm::cursor::Show
                )?;
                Ok(())
            }
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        let mut new_vector: Vec<Cell> = Vec::new();
        let space_to_allocate = (width as usize) * (height as usize);
        new_vector.resize_with(space_to_allocate as usize, Cell::default);

        self.cells = new_vector;
    }
}
