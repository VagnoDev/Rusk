use crossterm::{cursor::MoveTo, execute, queue, style::Print, terminal::{Clear, ClearType}};
use crossterm::terminal;
use std::io::{stdout, Write};
use crate::buffer::Buffer;
/* 
enum Colors {
    BLACK,
    WHITE
}

*/
    
pub fn draw(buffer: &mut Buffer) -> std::io::Result<()> {
    let mut stdout = stdout();
    let size = terminal::size();
    
    execute!(stdout, Clear(ClearType::All))?;

    for (index, line) in buffer.lines.iter().enumerate() {
        queue!(stdout, MoveTo(0, index as u16))?;
       queue!(stdout, Print(line))?;
    }
    stdout.flush()?;
    execute!(stdout, MoveTo(buffer.row as u16, buffer.column as u16))?;
    Ok(())
}
