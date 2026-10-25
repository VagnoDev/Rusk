use crate::editor::buffer::Buffer;

use std::{fs::write, path::Path};

pub fn save(buffer: &Buffer, path: &Path) -> std::io::Result<()> {
    let content = buffer.lines.join("\n");

    write(path, content)?;

    Ok(())
}
