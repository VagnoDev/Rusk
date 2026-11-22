use std::{io::stdout, path::PathBuf, time::Instant};

use crate::{
    Status::Continue,
    app::{
        messages::MessageLevel,
        mode::{Mode, set_command, set_normal, set_search},
    },
    commands::{executor, parser},
    editor::editing::{insert_backward, insert_newline, remove_char},
    explorer::tree::{Explorer, read_dir},
    gui::{display::Display, editor_view::update_viewport, status_bar::send_message},
    input::keyboard::keybinding,
    io::save::save,
};
use crossterm::{
    event::{
        self,
        Event::{self},
        KeyEventKind,
    },
    execute,
};

use std::path::Path;

use crate::normal::{
    apply::{apply, lines_target},
    motion::{Motion, MotionContext, Viewport},
    motions::{mark, search},
    operator::finalize_target,
    operator::{IndentDirection, Operator},
    parser::{Command, Key, Parsed, Parser, Scroll},
    resolver::{MotionResult, ResolveError, resolve},
    target::Target,
    target::{Position, cursor_after},
};
use crate::{
    app::actions::Action::{self},
    editor::buffer,
    editor::editing::{insert_char, insert_forward},
};
use crossterm::terminal;
use std::collections::VecDeque;

use crate::app::context;
use crate::context::EditorContext;
use crate::editor::buffer::Buffer;

mod app;
mod commands;
mod editor;
mod explorer;
mod gui;
mod input;
mod io;
mod normal;

pub struct App {
    buffers: Vec<OpenBuffer>,
    explorer: Explorer,
    sidebar_visible: bool,
    active_buffer: usize,
}

pub struct OpenBuffer {
    pub path: Option<PathBuf>,
    pub buffer: Buffer,
}

pub enum Status {
    Continue,
    Quit,
    OpenExplorer,
    CloseExplorer,
}

fn main() -> std::io::Result<()> {
    let path = std::env::args().nth(1);

    let mut app = match &path {
        Some(path) => match Path::new(&path).is_dir() {
            true => App {
                buffers: vec![OpenBuffer {
                    path: None,
                    buffer: Buffer {
                        lines: vec![String::new()],
                        row: 0,
                        column: 0,
                        modified: false,
                    },
                }],
                explorer: read_dir(&PathBuf::from(&path), 0),
                sidebar_visible: false,
                active_buffer: 0,
            },
            false => {
                let file_path = Path::new(&path);

                let parent = file_path
                    .parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .unwrap_or(Path::new("."));

                App {
                    buffers: vec![OpenBuffer {
                        path: Some(file_path.to_path_buf()),
                        buffer: buffer::fetch(path)?,
                    }],
                    explorer: read_dir(&PathBuf::from(parent), 0),
                    sidebar_visible: false,
                    active_buffer: 0,
                }
            }
        },
        None => App {
            buffers: vec![OpenBuffer {
                path: None,
                buffer: Buffer {
                    lines: vec![String::new()],
                    row: 0,
                    column: 0,
                    modified: false,
                },
            }],
            explorer: read_dir(&PathBuf::from("."), 0),
            sidebar_visible: false,
            active_buffer: 0,
        },
    };

    let mut context = EditorContext {
        mode: Mode::Normal,
        command_buffer: String::new(),
        file_path: app.buffers[app.active_buffer].path.clone(),
        current_message: None,
        savedialog: false,
        quit_after_save: false,
        viewport_top: 0,
        search_forward: None,
        motion_state: crate::normal::motion::MotionState::default(),
        parser: Parser::default(),
        pending_space: false,
        register: None,
        search_line: None,
        pending_keys: String::new(),
    };

    terminal::enable_raw_mode()?;
    execute!(
        stdout(),
        terminal::EnterAlternateScreen,
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        crossterm::cursor::MoveTo(0, 0)
    )?;

    let mut display = Display::new()?;

    let mut queue = VecDeque::new();

    display.draw(
        &app.buffers[app.active_buffer].buffer,
        &context,
        &app.explorer,
        app.sidebar_visible,
        &app.buffers,
    )?;

    let mut needs_redraw = true;

    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press
                        && let Some(realbind) = keybinding(key, &context.mode)
                    {
                        queue.push_back(realbind);
                        needs_redraw = true;
                    }
                }
                Event::Resize(width, height) => {
                    display.resize(width as u32, height as u32);
                    needs_redraw = true;
                }
                _ => {}
            }
        }
        let expired = match &context.current_message {
            Some(message) => Instant::now() >= message.duration,
            None => false,
        };
        if expired {
            context.current_message = None;
            needs_redraw = true;
        }
        let status = manager(
            &mut queue,
            &mut app.buffers,
            &mut app.active_buffer,
            &mut context,
            &mut app.explorer,
            (display.height.saturating_sub(1)) as usize,
        )?;

        match status {
            Status::Continue => {}
            Status::OpenExplorer => {
                app.sidebar_visible = true;
                context.mode = Mode::Explorer
            }
            Status::CloseExplorer => {
                app.sidebar_visible = false;
                context.mode = Mode::Normal
            }
            Status::Quit => break,
        }
        if needs_redraw {
            display.draw(
                &app.buffers[app.active_buffer].buffer,
                &context,
                &app.explorer,
                app.sidebar_visible,
                &app.buffers,
            )?;

            needs_redraw = false;
        }
    }
    terminal::disable_raw_mode()?;
    execute!(stdout(), terminal::LeaveAlternateScreen)?;
    Ok(())
}

fn apply_cursor(buffer: &mut Buffer, position: Position) {
    buffer.column = position.line;
    buffer.row = position.col;
}

fn sync_viewport(context: &mut EditorContext, editor_height: usize) {
    context.motion_state.viewport = Viewport {
        top: context.viewport_top,
        height: editor_height,
    };
}

fn report(context: &mut EditorContext, error: ResolveError, missing: &str) {
    if error == ResolveError::Failed {
        send_message(context, missing.to_string(), MessageLevel::Error);
    }
}

fn scroll(
    buffer: &mut Buffer,
    context: &mut EditorContext,
    kind: Scroll,
    count: usize,
    editor_height: usize,
) {
    let last = buffer.line_count().saturating_sub(1);
    let half = (editor_height / 2).max(1);
    let line = buffer.column;
    let (top, target) = match kind {
        Scroll::PageDown => {
            let top = (context.viewport_top + editor_height * count).min(last);
            (top, top)
        }
        Scroll::PageUp => {
            let top = context.viewport_top.saturating_sub(editor_height * count);
            (top, (top + editor_height - 1).min(last).min(line))
        }
        Scroll::HalfDown => {
            let step = half * count;
            (
                (context.viewport_top + step).min(last),
                (line + step).min(last),
            )
        }
        Scroll::HalfUp => {
            let step = half * count;
            (
                context.viewport_top.saturating_sub(step),
                line.saturating_sub(step),
            )
        }
    };
    context.viewport_top = top;
    context.motion_state.desired = None;
    let len = buffer.lines[target].chars().count();
    apply_cursor(
        buffer,
        Position::new(target, buffer.row.min(len.saturating_sub(1))),
    );
}

fn run_command(
    buffer: &mut Buffer,
    context: &mut EditorContext,
    command: Command,
    editor_height: usize,
) {
    sync_viewport(context, editor_height);
    let from = Position::new(buffer.column, buffer.row);

    match command {
        Command::Motion {
            count,
            operator: None,
            motion,
            ..
        } => match resolve(
            buffer,
            from,
            motion,
            count,
            MotionContext::Standalone,
            &mut context.motion_state,
        ) {
            Ok(MotionResult::Cursor(target)) => apply_cursor(buffer, target),
            Ok(MotionResult::Operator(_)) => {}
            Err(error) => {
                let missing = match motion {
                    Motion::Mark(_) | Motion::MarkLine(_) => "Mark not set",
                    _ => "Pattern not found",
                };
                report(context, error, missing);
            }
        },
        Command::Motion {
            count,
            operator: Some(operator),
            direction,
            motion,
        } => {
            let result = resolve(
                buffer,
                from,
                motion,
                count,
                MotionContext::WithOperator(operator),
                &mut context.motion_state,
            );
            if let Ok(MotionResult::Operator(target)) = result
                && let Ok(target) = finalize_target(buffer, operator, target)
            {
                finish_operator(buffer, context, operator, direction, target, from);
            }
        }
        Command::Lines {
            count,
            operator,
            direction,
        } => {
            let target = lines_target(buffer, from, count);
            finish_operator(buffer, context, operator, direction, target, from);
        }
        Command::SetMark(name) => {
            if mark::set(&mut context.motion_state, name, from).is_err() {
                send_message(context, "Invalid mark".to_string(), MessageLevel::Error);
            }
        }
        Command::Search { forward } => {
            context.current_message = None;
            set_search(context, forward);
        }
        Command::Scroll { kind, count } => {
            scroll(buffer, context, kind, count.unwrap_or(1), editor_height)
        }
    }
}

fn finish_operator(
    buffer: &mut Buffer,
    context: &mut EditorContext,
    operator: Operator,
    direction: Option<IndentDirection>,
    target: Target,
    from: Position,
) {
    let applied = apply(
        buffer,
        &mut context.motion_state,
        operator,
        direction,
        target,
        from,
    );
    apply_cursor(buffer, applied.cursor);
    if applied.register.is_some() {
        context.register = applied.register;
    }
    if applied.insert {
        context.mode = Mode::Insert;
    }
}

fn normal_key(
    buffer: &mut Buffer,
    context: &mut EditorContext,
    key: Key,
    editor_height: usize,
) -> Option<Status> {
    if context.parser.is_idle() {
        if context.pending_space {
            context.pending_space = false;
            context.pending_keys.clear();
            if key == Key::Char('e') {
                return Some(Status::OpenExplorer);
            }
            return None;
        }
        match key {
            Key::Char(' ') => {
                context.pending_space = true;
                context.pending_keys = " ".to_string();
                return None;
            }
            Key::Char('a') => {
                insert_forward(buffer, context);
                return None;
            }
            Key::Char('i') => {
                insert_backward(context);
                return None;
            }
            Key::Char(':') => {
                context.current_message = None;
                set_command(context);
                return None;
            }
            _ => {}
        }
    }
    match context.parser.feed(key) {
        Parsed::Incomplete => match key {
            Key::Char(c) => context.pending_keys.push(c),
            Key::Ctrl(c) => context
                .pending_keys
                .push_str(&format!("^{}", c.to_ascii_uppercase())),
            _ => {}
        },
        Parsed::Complete(command) => {
            context.pending_keys.clear();
            run_command(buffer, context, command, editor_height);
        }
        _ => context.pending_keys.clear(),
    }
    None
}

fn run_search(
    buffer: &mut Buffer,
    context: &mut EditorContext,
    pattern: &str,
    forward: bool,
    editor_height: usize,
) {
    sync_viewport(context, editor_height);
    let position = Position::new(buffer.column, buffer.row);

    match search::start(
        buffer,
        position,
        pattern,
        forward,
        1,
        &mut context.motion_state,
    ) {
        Ok(output) => {
            mark::record_jump(&mut context.motion_state, position);
            apply_cursor(buffer, cursor_after(buffer, &output));
        }
        Err(error) => report(context, error, "Pattern not found"),
    }
}

fn manager(
    queue: &mut VecDeque<Action>,
    buffers: &mut Vec<OpenBuffer>,
    active_buffer: &mut usize,
    context: &mut EditorContext,
    explorer: &mut Explorer,
    editor_height: usize,
) -> std::io::Result<Status> {
    let Some(first) = queue.pop_front() else {
        return Ok(Continue);
    };

    match context.mode {
        Mode::Normal => {
            let lines = &mut buffers[*active_buffer].buffer;
            if let Action::NormalKey(key) = first
                && let Some(status) = normal_key(lines, context, key, editor_height)
            {
                return Ok(status);
            }
        }
        Mode::Insert => {
            let lines = &mut buffers[*active_buffer].buffer;
            match first {
                Action::InsertChar(char) => insert_char(char, lines),
                Action::SetNormal => {
                    set_normal(context);
                    lines.row = lines.row.saturating_sub(1);
                }
                Action::RemoveChar => remove_char(lines),
                Action::InserNewLine => insert_newline(lines),
                _ => {}
            }
        }
        Mode::Command => {
            let lines = &mut buffers[*active_buffer].buffer;
            match first {
                Action::EscCommand => set_normal(context),
                Action::InsertChar(char) => context.command_buffer.push(char),
                Action::RemoveChar => {
                    context.command_buffer.pop();
                }
                Action::SendCommand => {
                    if let Some(forward) = context.search_forward {
                        let pattern = context.command_buffer.clone();
                        let prefix = if forward { '/' } else { '?' };
                        set_normal(context);
                        context.search_line = Some(format!("{prefix}{pattern}"));
                        run_search(lines, context, &pattern, forward, editor_height);
                        return Ok(Status::Continue);
                    }

                    if context.savedialog {
                        let path = std::path::Path::new(&context.command_buffer);

                        match save(lines, path) {
                            Ok(()) => {
                                context.file_path = Some(path.to_path_buf());
                                buffers[*active_buffer].path = Some(path.to_path_buf());
                                context.command_buffer.clear();
                                context.savedialog = false;
                                set_normal(context);

                                if context.quit_after_save {
                                    context.quit_after_save = false;
                                    return Ok(Status::Quit);
                                }
                                send_message(
                                    context,
                                    "File Saved.".to_string(),
                                    MessageLevel::Success,
                                );
                                return Ok(Status::Continue);
                            }
                            Err(error) => {
                                send_message(
                                    context,
                                    format!("Cannot save file: {error}"),
                                    MessageLevel::Error,
                                );
                                return Ok(Status::Continue);
                            }
                        }
                    }

                    let command = parser::parse(context.command_buffer.as_str());
                    let needs_filename = matches!(
                        &command,
                        parser::Command::Write(None) | parser::Command::WriteQuit(None)
                    );

                    if needs_filename && context.file_path.is_none() {
                        context.quit_after_save =
                            matches!(&command, parser::Command::WriteQuit(None));

                        context.savedialog = true;
                        context.command_buffer.clear();
                        return Ok(Status::Continue);
                    }

                    if matches!(&command, parser::Command::Unknown) {
                        let message = format!("Not an editor command: {}", context.command_buffer);
                        set_normal(context);
                        send_message(context, message, MessageLevel::Error);
                        return Ok(Status::Continue);
                    }

                    let is_write = matches!(&command, parser::Command::Write(_));
                    let is_saving = matches!(
                        &command,
                        parser::Command::Write(_) | parser::Command::WriteQuit(_)
                    );
                    let status = executor::execute(command, lines, &mut context.file_path)?;

                    if is_saving {
                        buffers[*active_buffer].path = context.file_path.clone();
                    }

                    if is_write {
                        set_normal(context);
                        send_message(context, "File Saved.".to_string(), MessageLevel::Success);
                        buffers[*active_buffer].path = context.file_path.clone();
                    }

                    return Ok(status);
                }
                _ => {}
            }
        }
        Mode::Explorer => match first {
            Action::SetNormal => {
                set_normal(context);
                return Ok(Status::CloseExplorer);
            }
            Action::ExplorerDown => explorer.explorer_down(),
            Action::ExplorerUp => explorer.explorer_up(),
            Action::EnterFile => {
                if explorer.entries[explorer.selected].is_dir {
                    explorer.explorer_toggle()
                } else {
                    let path = explorer.entries[explorer.selected].path.clone();

                    let existing = buffers
                        .iter()
                        .position(|open_buffer| open_buffer.path.as_ref() == Some(&path));

                    let opened = match existing {
                        Some(index) => {
                            *active_buffer = index;
                            true
                        }
                        None => match buffer::fetch(path.to_string_lossy().as_ref()) {
                            Ok(buffer) => {
                                buffers.push(OpenBuffer {
                                    path: Some(path.clone()),
                                    buffer,
                                });
                                *active_buffer = buffers.len() - 1;
                                true
                            }
                            Err(error) => {
                                send_message(
                                    context,
                                    format!("Cannot open file: {error}"),
                                    MessageLevel::Error,
                                );
                                false
                            }
                        },
                    };
                    if opened {
                        context.file_path = Some(path.clone());
                    }
                }
            }
            _ => {}
        },
    }
    update_viewport(&buffers[*active_buffer].buffer, context, editor_height);
    Ok(Continue)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup(text: &[&str]) -> (Buffer, EditorContext) {
        let buffer = Buffer {
            lines: text.iter().map(|s| s.to_string()).collect(),
            row: 0,
            column: 0,
            modified: false,
        };
        let context = EditorContext {
            mode: Mode::Normal,
            command_buffer: String::new(),
            file_path: None,
            current_message: None,
            savedialog: false,
            quit_after_save: false,
            viewport_top: 0,
            search_forward: None,
            motion_state: crate::normal::motion::MotionState::default(),
            parser: Parser::default(),
            pending_space: false,
            register: None,
            search_line: None,
            pending_keys: String::new(),
        };
        (buffer, context)
    }

    fn keys(buffer: &mut Buffer, context: &mut EditorContext, input: &str) {
        for c in input.chars() {
            normal_key(buffer, context, Key::Char(c), 10);
        }
    }

    fn at(buffer: &Buffer) -> (usize, usize) {
        (buffer.column, buffer.row)
    }

    fn sample() -> Vec<&'static str> {
        vec!["alpha beta gamma", "  delta", "", "epsilon zeta", "eta"]
    }

    #[test]
    fn word_motions_move_cursor() {
        let (mut b, mut c) = setup(&sample());
        keys(&mut b, &mut c, "w");
        assert_eq!(at(&b), (0, 6));
        keys(&mut b, &mut c, "2w");
        assert_eq!(at(&b), (1, 2));
        keys(&mut b, &mut c, "b");
        assert_eq!(at(&b), (0, 11));
        keys(&mut b, &mut c, "e");
        assert_eq!(at(&b), (0, 15));
    }

    #[test]
    fn line_motions() {
        let (mut b, mut c) = setup(&sample());
        keys(&mut b, &mut c, "$");
        assert_eq!(at(&b), (0, 15));
        keys(&mut b, &mut c, "0");
        assert_eq!(at(&b), (0, 0));
        keys(&mut b, &mut c, "j");
        keys(&mut b, &mut c, "^");
        assert_eq!(at(&b), (1, 2));
    }

    #[test]
    fn vertical_keeps_desired_column() {
        let (mut b, mut c) = setup(&sample());
        keys(&mut b, &mut c, "$");
        keys(&mut b, &mut c, "j");
        assert_eq!(at(&b), (1, 6));
        keys(&mut b, &mut c, "j");
        assert_eq!(at(&b), (2, 0));
        keys(&mut b, &mut c, "j");
        assert_eq!(at(&b), (3, 11));
    }

    #[test]
    fn goto_motions() {
        let (mut b, mut c) = setup(&sample());
        keys(&mut b, &mut c, "G");
        assert_eq!(at(&b).0, 4);
        keys(&mut b, &mut c, "1G");
        assert_eq!(at(&b).0, 0);
        keys(&mut b, &mut c, "3G");
        assert_eq!(at(&b).0, 2);
    }

    #[test]
    fn double_backtick_toggles() {
        let (mut b, mut c) = setup(&sample());
        keys(&mut b, &mut c, "G");
        assert_eq!(at(&b).0, 4);
        keys(&mut b, &mut c, "``");
        assert_eq!(at(&b), (0, 0));
        keys(&mut b, &mut c, "``");
        assert_eq!(at(&b).0, 4);
    }

    #[test]
    fn find_repeat_and_reverse() {
        let (mut b, mut c) = setup(&["a-b-c-d"]);
        keys(&mut b, &mut c, "f-");
        assert_eq!(at(&b), (0, 1));
        keys(&mut b, &mut c, ";");
        assert_eq!(at(&b), (0, 3));
        keys(&mut b, &mut c, ",");
        assert_eq!(at(&b), (0, 1));
        keys(&mut b, &mut c, "t-");
        assert_eq!(at(&b), (0, 2));
    }

    #[test]
    fn marks_set_and_jump() {
        let (mut b, mut c) = setup(&sample());
        keys(&mut b, &mut c, "w");
        keys(&mut b, &mut c, "ma");
        keys(&mut b, &mut c, "G");
        keys(&mut b, &mut c, "'a");
        assert_eq!(at(&b), (0, 0));
        keys(&mut b, &mut c, "G`a");
        assert_eq!(at(&b), (0, 6));
    }

    #[test]
    fn missing_mark_reports() {
        let (mut b, mut c) = setup(&sample());
        keys(&mut b, &mut c, "`z");
        assert_eq!(at(&b), (0, 0));
        assert!(c.current_message.is_some());
    }

    #[test]
    fn delete_word() {
        let (mut b, mut c) = setup(&sample());
        keys(&mut b, &mut c, "dw");
        assert_eq!(b.lines[0], "beta gamma");
        assert_eq!(at(&b), (0, 0));
        assert!(b.modified);
        assert_eq!(c.register.as_ref().unwrap().text, "alpha ");
    }

    #[test]
    fn delete_line_and_count() {
        let (mut b, mut c) = setup(&sample());
        keys(&mut b, &mut c, "dd");
        assert_eq!(b.lines.len(), 4);
        assert_eq!(b.lines[0], "  delta");
        assert_eq!(at(&b), (0, 2));
        keys(&mut b, &mut c, "2dd");
        assert_eq!(b.lines, vec!["epsilon zeta", "eta"]);
    }

    #[test]
    fn delete_lines_clamped_at_end() {
        let (mut b, mut c) = setup(&["a", "b", "c"]);
        keys(&mut b, &mut c, "j");
        keys(&mut b, &mut c, "9dd");
        assert_eq!(b.lines, vec!["a"]);
        assert_eq!(at(&b).0, 0);
    }

    #[test]
    fn change_word_enters_insert() {
        let (mut b, mut c) = setup(&sample());
        keys(&mut b, &mut c, "cw");
        assert_eq!(b.lines[0], " beta gamma");
        assert!(matches!(c.mode, Mode::Insert));
        assert_eq!(at(&b), (0, 0));
    }

    #[test]
    fn yank_line_keeps_buffer() {
        let (mut b, mut c) = setup(&sample());
        keys(&mut b, &mut c, "w");
        keys(&mut b, &mut c, "yy");
        assert_eq!(b.lines.len(), 5);
        assert!(!b.modified);
        assert_eq!(at(&b), (0, 6));
        let r = c.register.as_ref().unwrap();
        assert!(r.linewise);
        assert_eq!(r.text, "alpha beta gamma\n");
    }

    #[test]
    fn indent_lines() {
        let (mut b, mut c) = setup(&["a", "b"]);
        keys(&mut b, &mut c, ">>");
        assert_eq!(b.lines[0], "    a");
        keys(&mut b, &mut c, "<<");
        assert_eq!(b.lines[0], "a");
        keys(&mut b, &mut c, "2>>");
        assert_eq!(b.lines, vec!["    a", "    b"]);
    }

    #[test]
    fn operator_failure_cancels() {
        let (mut b, mut c) = setup(&["abc"]);
        keys(&mut b, &mut c, "dfz");
        assert_eq!(b.lines[0], "abc");
        keys(&mut b, &mut c, "x");
        assert!(c.parser.is_idle());
    }

    #[test]
    fn scroll_moves_viewport_and_cursor() {
        let text: Vec<String> = (0..50).map(|i| format!("line {i}")).collect();
        let refs: Vec<&str> = text.iter().map(|s| s.as_str()).collect();
        let (mut b, mut c) = setup(&refs);
        normal_key(&mut b, &mut c, Key::Ctrl('d'), 10);
        assert_eq!(c.viewport_top, 5);
        assert_eq!(at(&b).0, 5);
        normal_key(&mut b, &mut c, Key::Ctrl('f'), 10);
        assert_eq!(c.viewport_top, 15);
        assert_eq!(at(&b).0, 15);
        normal_key(&mut b, &mut c, Key::Ctrl('u'), 10);
        assert_eq!(c.viewport_top, 10);
        assert_eq!(at(&b).0, 10);
        normal_key(&mut b, &mut c, Key::Ctrl('b'), 10);
        assert_eq!(c.viewport_top, 0);
        assert!(at(&b).0 <= 9);
    }

    #[test]
    fn space_e_opens_explorer() {
        let (mut b, mut c) = setup(&["x"]);
        assert!(normal_key(&mut b, &mut c, Key::Char(' '), 10).is_none());
        assert!(matches!(
            normal_key(&mut b, &mut c, Key::Char('e'), 10),
            Some(Status::OpenExplorer)
        ));
    }

    #[test]
    fn colon_and_slash_switch_mode() {
        let (mut b, mut c) = setup(&["x"]);
        keys(&mut b, &mut c, ":");
        assert!(matches!(c.mode, Mode::Command));
        let (mut b, mut c) = setup(&["x"]);
        keys(&mut b, &mut c, "/");
        assert!(matches!(c.mode, Mode::Command));
        assert_eq!(c.search_forward, Some(true));
    }

    #[test]
    fn search_line_persists_after_confirm() {
        let (mut b, mut c) = setup(&["one two", "three two"]);
        keys(&mut b, &mut c, "/");
        c.command_buffer = "two".to_string();
        let forward = c.search_forward.unwrap();
        let pattern = c.command_buffer.clone();
        set_normal(&mut c);
        c.search_line = Some(format!("/{pattern}"));
        run_search(&mut b, &mut c, &pattern, forward, 10);
        assert_eq!(c.search_line.as_deref(), Some("/two"));
        keys(&mut b, &mut c, ":");
        assert!(c.search_line.is_none());
    }

    #[test]
    fn real_search_moves_cursor() {
        let (mut b, mut c) = setup(&["one two", "three two"]);
        run_search(&mut b, &mut c, "two", true, 10);
        assert_eq!(at(&b), (0, 4));
    }

    #[test]
    fn pending_keys_track_and_clear() {
        let (mut b, mut c) = setup(&["one two", "three"]);
        keys(&mut b, &mut c, "2");
        assert_eq!(c.pending_keys, "2");
        keys(&mut b, &mut c, "d");
        assert_eq!(c.pending_keys, "2d");
        keys(&mut b, &mut c, "2");
        assert_eq!(c.pending_keys, "2d2");
        keys(&mut b, &mut c, "w");
        assert_eq!(c.pending_keys, "");
    }

    #[test]
    fn pending_keys_clear_on_cancel() {
        let (mut b, mut c) = setup(&["one two"]);
        keys(&mut b, &mut c, "d");
        assert_eq!(c.pending_keys, "d");
        normal_key(&mut b, &mut c, Key::Esc, 10);
        assert_eq!(c.pending_keys, "");
    }

    #[test]
    fn insert_then_normal_keeps_keys_working() {
        let (b, mut c) = setup(&["one two", "three"]);
        let mut bufs = vec![OpenBuffer {
            path: None,
            buffer: b,
        }];
        let mut active = 0;
        let mut ex = Explorer {
            entries: vec![],
            selected: 0,
        };
        let mut q = VecDeque::new();
        let mut run = |a: Action, c: &mut EditorContext, bufs: &mut Vec<OpenBuffer>| {
            q.push_back(a);
            manager(&mut q, bufs, &mut active, c, &mut ex, 10).unwrap();
        };
        run(Action::NormalKey(Key::Char('c')), &mut c, &mut bufs);
        run(Action::NormalKey(Key::Char('c')), &mut c, &mut bufs);
        assert!(matches!(c.mode, Mode::Insert));
        run(Action::SetNormal, &mut c, &mut bufs);
        assert!(matches!(c.mode, Mode::Normal));
        run(Action::NormalKey(Key::Char('i')), &mut c, &mut bufs);
        assert!(matches!(c.mode, Mode::Insert));
        run(Action::SetNormal, &mut c, &mut bufs);
        run(Action::NormalKey(Key::Char('j')), &mut c, &mut bufs);
        assert_eq!(bufs[0].buffer.column, 1);
    }

    #[test]
    fn esc_from_insert_at_line_end_keeps_motions_working() {
        let (b, mut c) = setup(&["one two", "three"]);
        let mut bufs = vec![OpenBuffer {
            path: None,
            buffer: b,
        }];
        let mut active = 0;
        let mut ex = Explorer {
            entries: vec![],
            selected: 0,
        };
        let mut q = VecDeque::new();
        for a in [
            Action::NormalKey(Key::Char('$')),
            Action::NormalKey(Key::Char('a')),
            Action::InsertChar('z'),
            Action::SetNormal,
        ] {
            q.push_back(a);
            manager(&mut q, &mut bufs, &mut active, &mut c, &mut ex, 10).unwrap();
        }
        assert_eq!(bufs[0].buffer.row, 7);
        q.push_back(Action::NormalKey(Key::Char('b')));
        manager(&mut q, &mut bufs, &mut active, &mut c, &mut ex, 10).unwrap();
        assert_eq!(bufs[0].buffer.row, 4);
        q.push_back(Action::NormalKey(Key::Char('j')));
        manager(&mut q, &mut bufs, &mut active, &mut c, &mut ex, 10).unwrap();
        assert_eq!(bufs[0].buffer.column, 1);
    }
}
