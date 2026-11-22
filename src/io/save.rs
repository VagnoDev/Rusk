use crate::editor::buffer::{Buffer, LineEnding};

use std::{fs::write, path::Path};

pub fn save(buffer: &mut Buffer, path: &Path) -> std::io::Result<()> {
    let content = match buffer.ending {
        LineEnding::Lf => {
            buffer.lines.join("\n")
        },
        LineEnding::Crlf => {
            buffer.lines.join("\r\n")
        }
    };


    write(path, content)?;

    buffer.mark_saved();

    Ok(())
}
