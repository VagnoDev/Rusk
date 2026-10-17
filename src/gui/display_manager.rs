use crossterm::{cursor::MoveTo, execute, queue, style::{Color, Print, SetBackgroundColor, SetForegroundColor}, terminal::{Clear, ClearType}};
use crossterm::terminal;
use ratatui::style;
use std::{char, io::{Write, stdout}};
use crate::buffer::Buffer;

#[derive(Default)]
enum Colors {
    #[default]
    BLACK,
    WHITE,
    RED,
    GREEN,
    BLUE,
    CYAN
}

pub struct Cell {
    symbol: char,
    fg: Colors,
    bg: Colors
}

pub struct Display {
    cells: Vec<Cell>,
    width: u32,
    height: u32
}

fn terminal_to_color(color: &Colors) -> crossterm::style::Color {
    match color {
        Colors::BLACK => Color::Black,
        Colors::WHITE => Color::White,
        Colors::RED => Color::Red,
        Colors::GREEN => Color::Green,
        Colors::BLUE => Color::Blue,
        Colors::CYAN => Color::Cyan
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            symbol: ' ',
            fg: Colors::WHITE,
            bg: Colors::BLACK
        }
    }
}

impl Display {
    pub fn new() -> std::io::Result<Self> { 
        let (width, height) = terminal::size()?;
        let mut new_vector: Vec<Cell> = Vec::new();
        let space_to_allocate = (width as usize) * (height as usize);
        new_vector.resize_with(space_to_allocate as usize, Cell::default);

        return Ok(Self { cells: (new_vector), width: (width as u32), height: (height as u32) });
    }

    pub fn draw(&mut self, buffer: &mut Buffer) -> std::io::Result<()>{
        let mut stdout = stdout();

        execute!(stdout, crossterm::cursor::Hide)?;
        execute!(stdout, Clear(ClearType::All))?;

        for cell in self.cells.iter_mut() {
            *cell = Cell::default();
        }
    
        for (index, line) in buffer.lines.iter().enumerate() {
            for(col, char) in line.chars().enumerate(){
                
                let cell_index = index * self.width as usize + col;

                if let Some(cell) = self.cells.get_mut(cell_index) {
                    cell.symbol = char;
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
        execute!(
            stdout,
            MoveTo(buffer.row as u16, buffer.column as u16),
            crossterm::cursor::Show
        )?;
        Ok(())
    }
} 
   