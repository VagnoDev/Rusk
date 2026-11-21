use crate::{
    editor::buffer::Buffer,
    normal::{
        motion::MotionState,
        operator::{IndentDirection, Operator},
        scanner::{is_blank, line_len},
        target::{MotionKind, Position, Target},
    },
};

const INDENT: &str = "    ";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Register {
    pub text: String,
    pub linewise: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Applied {
    pub cursor: Position,
    pub insert: bool,
    pub register: Option<Register>,
}

fn byte_index(line: &str, col: usize) -> usize {
    line.char_indices()
        .nth(col)
        .map(|(index, _)| index)
        .unwrap_or(line.len())
}

fn first_non_blank(line: &str) -> usize {
    line.chars().take_while(|c| is_blank(*c)).count()
}

fn drop_lines(state: &mut MotionState, from: usize, count: usize) {
    state
        .marks
        .retain(|_, position| position.line < from || position.line >= from + count);
    for position in state.marks.values_mut() {
        if position.line >= from + count {
            position.line -= count;
        }
    }
}

fn clamp(buffer: &Buffer, position: Position) -> Position {
    let line = position.line.min(buffer.line_count().saturating_sub(1));
    let len = line_len(buffer, line).unwrap_or(0);
    Position::new(line, position.col.min(len.saturating_sub(1)))
}

fn lines_of(buffer: &Buffer, target: &Target) -> (usize, usize) {
    let first = target.start.line.min(buffer.line_count().saturating_sub(1));
    let end = target.end.line.min(buffer.line_count()).max(first + 1);
    (first, end)
}

fn yank_chars(buffer: &Buffer, target: &Target) -> String {
    let (start, end) = (target.start, target.end);
    if start.line == end.line {
        let line = &buffer.lines[start.line];
        return line[byte_index(line, start.col)..byte_index(line, end.col)].to_string();
    }
    let mut text = String::new();
    let first = &buffer.lines[start.line];
    text.push_str(&first[byte_index(first, start.col)..]);
    for line in &buffer.lines[start.line + 1..end.line] {
        text.push('\n');
        text.push_str(line);
    }
    text.push('\n');
    let last = &buffer.lines[end.line];
    text.push_str(&last[..byte_index(last, end.col)]);
    text
}

fn delete_chars(buffer: &mut Buffer, state: &mut MotionState, target: &Target) {
    let (start, end) = (target.start, target.end);
    let tail = {
        let last = &buffer.lines[end.line];
        last[byte_index(last, end.col)..].to_string()
    };
    let head = {
        let first = &buffer.lines[start.line];
        first[..byte_index(first, start.col)].to_string()
    };
    buffer.lines[start.line] = head + &tail;
    if end.line > start.line {
        buffer.lines.drain(start.line + 1..=end.line);
        drop_lines(state, start.line + 1, end.line - start.line);
    }
}

fn indent_lines(buffer: &mut Buffer, first: usize, end: usize, direction: IndentDirection) {
    for line in &mut buffer.lines[first..end] {
        match direction {
            IndentDirection::Increase => {
                if !line.is_empty() {
                    line.insert_str(0, INDENT);
                }
            }
            IndentDirection::Decrease => {
                if line.starts_with('\t') {
                    line.remove(0);
                } else {
                    let spaces = line
                        .chars()
                        .take(INDENT.len())
                        .take_while(|c| *c == ' ')
                        .count();
                    line.drain(..spaces);
                }
            }
        }
    }
}

pub fn apply(
    buffer: &mut Buffer,
    state: &mut MotionState,
    operator: Operator,
    direction: Option<IndentDirection>,
    target: Target,
    from: Position,
) -> Applied {
    state.desired = None;
    let linewise = target.kind == MotionKind::Line;

    if operator == Operator::Indent {
        let (first, end) = lines_of(buffer, &target);
        indent_lines(
            buffer,
            first,
            end,
            direction.unwrap_or(IndentDirection::Increase),
        );
        buffer.mark_modified();
        let col = first_non_blank(&buffer.lines[first]);
        return Applied {
            cursor: clamp(buffer, Position::new(first, col)),
            insert: false,
            register: None,
        };
    }

    if linewise {
        let (first, end) = lines_of(buffer, &target);
        let mut text = buffer.lines[first..end].join("\n");
        text.push('\n');
        let register = Some(Register {
            text,
            linewise: true,
        });

        return match operator {
            Operator::Yank => {
                let cursor = if first < from.line {
                    Position::new(first, from.col)
                } else {
                    from
                };
                Applied {
                    cursor: clamp(buffer, cursor),
                    insert: false,
                    register,
                }
            }
            Operator::Delete => {
                buffer.lines.drain(first..end);
                drop_lines(state, first, end - first);
                if buffer.lines.is_empty() {
                    buffer.lines.push(String::new());
                }
                buffer.mark_modified();
                let line = first.min(buffer.line_count() - 1);
                let col = first_non_blank(&buffer.lines[line]);
                Applied {
                    cursor: clamp(buffer, Position::new(line, col)),
                    insert: false,
                    register,
                }
            }
            _ => {
                buffer.lines.drain(first + 1..end);
                drop_lines(state, first + 1, end - first - 1);
                buffer.lines[first].clear();
                buffer.mark_modified();
                Applied {
                    cursor: Position::new(first, 0),
                    insert: true,
                    register,
                }
            }
        };
    }

    let register = Some(Register {
        text: yank_chars(buffer, &target),
        linewise: false,
    });

    match operator {
        Operator::Yank => Applied {
            cursor: clamp(buffer, target.start),
            insert: false,
            register,
        },
        Operator::Delete => {
            delete_chars(buffer, state, &target);
            buffer.mark_modified();
            Applied {
                cursor: clamp(buffer, target.start),
                insert: false,
                register,
            }
        }
        _ => {
            delete_chars(buffer, state, &target);
            buffer.mark_modified();
            Applied {
                cursor: target.start,
                insert: true,
                register,
            }
        }
    }
}

pub fn lines_target(buffer: &Buffer, from: Position, count: Option<usize>) -> Target {
    let last = buffer.line_count().saturating_sub(1);
    let span = count.unwrap_or(1).max(1);
    Target::lines(from.line, from.line.saturating_add(span - 1).min(last))
}
