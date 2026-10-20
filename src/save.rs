use crate::buffer::Buffer;

use std::fs::write;


pub fn save(buffer: &Buffer, path: &String) -> std::io::Result<()> {
    let content = buffer.lines.join("\n");

    write(path, content)?;

    Ok(())

} 