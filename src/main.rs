
#[cfg(test)]
mod tests {
    use super::*;

    fn setup(text: &[&str]) -> (Buffer, EditorContext) {
        let buffer = Buffer {
            lines: text.iter().map(|s| s.to_string()).collect(),
            row: 0,
            column: 0,
            modified: false,
            ending: crate::editor::buffer::LineEnding::Lf,
            final_newline: false,
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

    fn temp_file(name: &str, content: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("rusk_{}_{name}", std::process::id()));
        std::fs::write(&path, content).unwrap();
        path
    }

    fn open(path: &Path) -> (Vec<OpenBuffer>, EditorContext) {
        let (_, mut c) = setup(&[""]);
        c.file_path = Some(path.to_path_buf());
        let buffers = vec![OpenBuffer {
            path: Some(path.to_path_buf()),
            buffer: buffer::fetch(path.to_str().unwrap()).unwrap(),
        }];
        (buffers, c)
    }

    #[test]
    fn parse_edit_commands() {
        assert_eq!(parser::parse("e"), parser::Command::Edit(None));
        assert_eq!(
            parser::parse("e foo.txt"),
            parser::Command::Edit(Some("foo.txt".to_string()))
        );
        assert_eq!(parser::parse("e!"), parser::Command::ForceEdit(None));
    }

    #[test]
    fn edit_refuses_modified_buffer() {
        let path = temp_file("edit_refuses", "one\ntwo\n");
        let (mut buffers, mut c) = open(&path);
        let mut active = 0;
        buffers[0].buffer.lines[0] = "changed".to_string();
        buffers[0].buffer.modified = true;
        edit(&mut buffers, &mut active, &mut c, None, false, 10);
        assert_eq!(buffers[0].buffer.lines[0], "changed");
        assert!(buffers[0].buffer.modified);
        assert!(matches!(c.mode, Mode::Normal));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn force_edit_reloads_from_disk() {
        let path = temp_file("force_reload", "one\ntwo\n");
        let (mut buffers, mut c) = open(&path);
        let mut active = 0;
        buffers[0].buffer.lines.push("three".to_string());
        buffers[0].buffer.column = 2;
        buffers[0].buffer.row = 4;
        buffers[0].buffer.modified = true;
        edit(&mut buffers, &mut active, &mut c, None, true, 10);
        assert_eq!(buffers[0].buffer.lines, vec!["one", "two"]);
        assert!(!buffers[0].buffer.modified);
        assert_eq!(at(&buffers[0].buffer), (1, 2));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn edit_opens_other_file() {
        let path = temp_file("edit_first", "one\n");
        let other = temp_file("edit_other", "other\n");
        let (mut buffers, mut c) = open(&path);
        let mut active = 0;
        edit(
            &mut buffers,
            &mut active,
            &mut c,
            Some(other.to_string_lossy().to_string()),
            false,
            10,
        );
        assert_eq!(active, 1);
        assert_eq!(buffers[1].buffer.lines, vec!["other"]);
        assert_eq!(c.file_path, Some(other.clone()));
        edit(
            &mut buffers,
            &mut active,
            &mut c,
            Some(path.to_string_lossy().to_string()),
            false,
            10,
        );
        assert_eq!(active, 0);
        assert_eq!(buffers.len(), 2);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_file(other).unwrap();
    }

    #[test]
    fn force_edit_other_file_discards_changes() {
        let path = temp_file("force_first", "one\n");
        let other = temp_file("force_other", "other\n");
        let (mut buffers, mut c) = open(&path);
        let mut active = 0;
        buffers[0].buffer.lines[0] = "changed".to_string();
        buffers[0].buffer.modified = true;
        edit(
            &mut buffers,
            &mut active,
            &mut c,
            Some(other.to_string_lossy().to_string()),
            true,
            10,
        );
        assert_eq!(active, 1);
        assert_eq!(buffers[0].buffer.lines, vec!["one"]);
        assert!(!buffers[0].buffer.modified);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_file(other).unwrap();
    }

    #[test]
    fn edit_without_file_name_fails() {
        let (b, mut c) = setup(&["text"]);
        let mut buffers = vec![OpenBuffer {
            path: None,
            buffer: b,
        }];
        let mut active = 0;
        edit(&mut buffers, &mut active, &mut c, None, false, 10);
        assert_eq!(buffers.len(), 1);
        assert_eq!(buffers[0].buffer.lines, vec!["text"]);
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

    #[test]
    fn unsaved_changes_checks_every_buffer() {
        let path = temp_file("unsaved_first", "one\n");
        let other = temp_file("unsaved_other", "other\n");
        let (mut buffers, mut c) = open(&path);
        buffers.push(OpenBuffer {
            path: Some(other.clone()),
            buffer: buffer::fetch(other.to_str().unwrap()).unwrap(),
        });
        assert!(!unsaved_changes(&buffers, 0, &mut c));
        buffers[1].buffer.modified = true;
        assert!(unsaved_changes(&buffers, 0, &mut c));
        std::fs::remove_file(path).unwrap();
        std::fs::remove_file(other).unwrap();
    }

    #[test]
    fn write_other_file_keeps_modified() {
        let path = temp_file("write_first", "one\n");
        let other = temp_file("write_other", "");
        let (mut buffers, mut c) = open(&path);
        buffers[0].buffer.modified = true;
        let command = parser::Command::Write(Some(other.to_string_lossy().to_string()));
        executor::execute(command, &mut buffers[0].buffer, &mut c).unwrap();
        assert!(buffers[0].buffer.modified);
        assert_eq!(c.file_path, Some(path.clone()));
        std::fs::remove_file(path).unwrap();
        std::fs::remove_file(other).unwrap();
    }

    #[test]
    fn write_names_unnamed_buffer() {
        let other = temp_file("write_unnamed", "");
        let (mut b, mut c) = setup(&["text"]);
        b.modified = true;
        let command = parser::Command::Write(Some(other.to_string_lossy().to_string()));
        executor::execute(command, &mut b, &mut c).unwrap();
        assert!(!b.modified);
        assert_eq!(c.file_path, Some(other.clone()));
        std::fs::remove_file(other).unwrap();
    }

    #[test]
    fn write_error_keeps_buffer() {
        let (mut b, mut c) = setup(&["text"]);
        b.modified = true;
        let command = parser::Command::Write(Some("/rusk_missing_dir/file".to_string()));
        assert!(executor::execute(command, &mut b, &mut c).is_err());
        assert!(b.modified);
    }
}
