use crate::{
    OpenBuffer,
    explorer::tree::{Entry, Explorer},
};

use super::display::{Colors, Display};

const BACKGROUND: Colors = Colors::Rgb(24, 24, 37);
const SELECTED: Colors = Colors::Rgb(49, 50, 68);
const GUIDE: Colors = Colors::Rgb(69, 71, 90);
const FOLDER: Colors = Colors::Rgb(137, 180, 250);
const TEXT: Colors = Colors::Rgb(205, 214, 244);
const MODIFIED: Colors = Colors::Rgb(249, 226, 175);

pub(super) fn draw_explorer(display: &mut Display, explorer: &Explorer, buffers: &[OpenBuffer]) {
    let width = display.width as usize;
    let height = display.height as usize;
    let sidebar_width = 25;
    let nerd_font = std::env::var_os("RUSK_NERD_FONT").is_some();

    for y in 0..height.saturating_sub(1) {
        let is_selected = y == explorer.selected && y < explorer.entries.len();

        for x in 0..sidebar_width.min(width) {
            if let Some(cell) = display.cells.get_mut(y * width + x) {
                cell.bg = if is_selected { SELECTED } else { BACKGROUND };
            }
        }
    }

    for (y, entry) in explorer.entries.iter().enumerate() {
        if y >= height.saturating_sub(1) {
            break;
        }

        let is_modified = buffers.iter().any(|open_buffer| {
            open_buffer.path.as_ref() == Some(&entry.path) && open_buffer.buffer.modified
        });

        let mut row: Vec<(char, Colors)> = vec![(' ', TEXT)];

        for level in 1..=entry.depth {
            let continues = has_next_sibling(&explorer.entries, y, level);
            let guide = match (level == entry.depth, continues) {
                (true, true) => '├',
                (true, false) => '└',
                (false, true) => '│',
                (false, false) => ' ',
            };
            row.push((guide, GUIDE));
            row.push((' ', GUIDE));
        }

        let (icon, icon_color) = file_icon(&entry.name);

        if entry.is_dir {
            let (chevron, folder) = match (entry.expanded, nerd_font) {
                (true, true) => ('\u{f107}', '\u{f07c}'),
                (false, true) => ('\u{f105}', '\u{f07b}'),
                (true, false) => ('▾', ' '),
                (false, false) => ('▸', ' '),
            };
            row.push((chevron, GUIDE));
            if nerd_font {
                row.push((' ', TEXT));
                row.push((folder, FOLDER));
            }
        } else if nerd_font {
            row.push((' ', TEXT));
            row.push((' ', TEXT));
            row.push((icon, icon_color));
        } else {
            row.push((' ', TEXT));
        }

        row.push((' ', TEXT));

        let name_color = if is_modified {
            MODIFIED
        } else if entry.is_dir {
            FOLDER
        } else {
            TEXT
        };

        for ch in entry.name.chars() {
            row.push((ch, name_color));
        }

        let limit = sidebar_width.min(width);

        if is_modified && limit >= 2 {
            row.truncate(limit - 2);
            while row.len() < limit - 2 {
                row.push((' ', TEXT));
            }
            row.push(('●', MODIFIED));
        }

        for (x, (ch, color)) in row.into_iter().take(limit).enumerate() {
            if let Some(cell) = display.cells.get_mut(y * width + x) {
                cell.symbol = ch;
                cell.fg = color;
            }
        }
    }

    let separator_x = sidebar_width;

    if separator_x < width {
        for y in 0..height.saturating_sub(1) {
            let cell_index = y * width + separator_x;

            if let Some(cell) = display.cells.get_mut(cell_index) {
                cell.symbol = '│';
                cell.fg = GUIDE;
                cell.bg = BACKGROUND;
            }
        }
    }
}

fn has_next_sibling(entries: &[Entry], index: usize, level: usize) -> bool {
    for entry in entries.iter().skip(index + 1) {
        if entry.depth < level {
            return false;
        }
        if entry.depth == level {
            return true;
        }
    }
    false
}

fn file_icon(name: &str) -> (char, Colors) {
    let lower = name.to_lowercase();
    let extension = lower.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("");

    match (lower.as_str(), extension) {
        (".gitignore" | ".gitmodules" | ".gitattributes", _) => {
            ('\u{e702}', Colors::Rgb(241, 80, 47))
        }
        ("license" | "license.md" | "license.txt", _) => ('\u{f0e3}', Colors::Rgb(203, 166, 247)),
        ("dockerfile", _) => ('\u{f308}', Colors::Rgb(69, 142, 230)),
        (_, "rs") => ('\u{e7a8}', Colors::Rgb(250, 179, 135)),
        (_, "ts" | "tsx") => ('\u{e628}', Colors::Rgb(81, 154, 186)),
        (_, "js" | "jsx" | "mjs" | "cjs") => ('\u{e74e}', Colors::Rgb(249, 226, 175)),
        (_, "json") => ('\u{e60b}', Colors::Rgb(249, 226, 175)),
        (_, "md") => ('\u{e609}', Colors::Rgb(137, 180, 250)),
        (_, "toml" | "yaml" | "yml" | "ini" | "conf") => ('\u{e615}', Colors::Rgb(166, 173, 200)),
        (_, "lock") => ('\u{f023}', Colors::Rgb(166, 173, 200)),
        (_, "py") => ('\u{e606}', Colors::Rgb(249, 226, 175)),
        (_, "go") => ('\u{e626}', Colors::Rgb(116, 199, 236)),
        (_, "c" | "h") => ('\u{e61e}', Colors::Rgb(137, 180, 250)),
        (_, "cpp" | "cc" | "hpp") => ('\u{e61d}', Colors::Rgb(137, 180, 250)),
        (_, "java") => ('\u{e738}', Colors::Rgb(243, 139, 168)),
        (_, "html") => ('\u{e736}', Colors::Rgb(250, 179, 135)),
        (_, "css" | "scss") => ('\u{e749}', Colors::Rgb(137, 180, 250)),
        (_, "sh" | "bash" | "zsh") => ('\u{f489}', Colors::Rgb(166, 227, 161)),
        (_, "lua") => ('\u{e620}', Colors::Rgb(137, 180, 250)),
        (_, "png" | "jpg" | "jpeg" | "gif" | "svg" | "ico") => {
            ('\u{f1c5}', Colors::Rgb(203, 166, 247))
        }
        (_, "txt") => ('\u{f15c}', TEXT),
        _ => ('\u{f15b}', Colors::Rgb(166, 173, 200)),
    }
}
