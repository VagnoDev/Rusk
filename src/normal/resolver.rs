use crate::{
    editor::buffer::Buffer,
    normal::{
        motion::{Desired, FindSpec, Motion, MotionContext, MotionState},
        motions::{
            bracket, file, find, horizontal, line, mark, paragraph,
            screen::{self, ScreenPos},
            search, sentence, vertical, word,
        },
        operator::finalize_target,
        scanner::line_len,
        target::{Position, Target, build_target, cursor_after},
    },
};

#[derive(Debug, PartialEq, Eq)]
pub enum ResolveError {
    InvalidPosition,
    InvalidCount,
    Failed,
}

pub type ResolveResult<T> = Result<T, ResolveError>;

pub enum MotionResult {
    Cursor(Position),
    Operator(Target),
}

pub fn validate_position(buffer: &Buffer, position: Position) -> ResolveResult<()> {
    let len = line_len(buffer, position.line).ok_or(ResolveError::InvalidPosition)?;

    if position.col < len || (len == 0 && position.col == 0) {
        Ok(())
    } else {
        Err(ResolveError::InvalidPosition)
    }
}

fn validate_count(count: Option<usize>) -> ResolveResult<()> {
    if count == Some(0) {
        return Err(ResolveError::InvalidCount);
    }

    Ok(())
}

fn find_char(
    buffer: &Buffer,
    position: Position,
    ch: char,
    forward: bool,
    till: bool,
    count: usize,
    state: &mut MotionState,
) -> ResolveResult<crate::normal::target::MotionOutput> {
    let spec = FindSpec { ch, forward, till };
    find::find(buffer, position, spec, count, state)
}

pub fn resolve(
    buffer: &Buffer,
    position: Position,
    motion: Motion,
    count: Option<usize>,
    motioncontext: MotionContext,
    state: &mut MotionState,
) -> ResolveResult<MotionResult> {
    validate_position(buffer, position)?;
    validate_count(count)?;

    let count1 = count.unwrap_or(1);
    let operator = motioncontext.operator();

    let output = match motion {
        Motion::Left => horizontal::left(position, count1)?,
        Motion::Right => horizontal::right(buffer, position, count1, operator.is_some())?,
        Motion::Up => vertical::resolve(buffer, position, count1, false, state)?,
        Motion::Down => vertical::resolve(buffer, position, count1, true, state)?,
        Motion::Word
        | Motion::WordBack
        | Motion::WordEnd
        | Motion::WordEndBack
        | Motion::BigWord
        | Motion::BigWordBack
        | Motion::BigWordEnd
        | Motion::BigWordEndBack => word::resolve(buffer, position, motion, count1, operator)?,
        Motion::LineStart => line::start(position),
        Motion::FirstNonBlank => line::first_non_blank(buffer, position)?,
        Motion::LineEnd => line::end(buffer, position, count1)?,
        Motion::LastNonBlank => line::last_non_blank(buffer, position, count1)?,
        Motion::ColumnGoto => line::column(buffer, position, count1)?,
        Motion::NextLineFirstNonBlank => line::next_line(buffer, position, count1)?,
        Motion::PrevLineFirstNonBlank => line::prev_line(buffer, position, count1)?,
        Motion::CurrentLineFirstNonBlank => line::current_line(buffer, position, count1)?,
        Motion::FileStart => file::goto(buffer, count, false)?,
        Motion::FileEnd => file::goto(buffer, count, true)?,
        Motion::FindChar(ch) => find_char(buffer, position, ch, true, false, count1, state)?,
        Motion::FindCharBack(ch) => find_char(buffer, position, ch, false, false, count1, state)?,
        Motion::TillChar(ch) => find_char(buffer, position, ch, true, true, count1, state)?,
        Motion::TillCharBack(ch) => find_char(buffer, position, ch, false, true, count1, state)?,
        Motion::RepeatFind => find::repeat(buffer, position, count1, false, state)?,
        Motion::RepeatFindReverse => find::repeat(buffer, position, count1, true, state)?,
        Motion::MatchBracket => match count {
            None => bracket::resolve(buffer, position)?,
            Some(n) => file::percent(buffer, n)?,
        },
        Motion::ParagraphForward => paragraph::resolve(buffer, position, count1, true)?,
        Motion::ParagraphBackward => paragraph::resolve(buffer, position, count1, false)?,
        Motion::SentenceForward => sentence::resolve(buffer, position, count1, true)?,
        Motion::SentenceBackward => sentence::resolve(buffer, position, count1, false)?,
        Motion::ScreenTop => screen::resolve(buffer, ScreenPos::Top, count1, state.viewport)?,
        Motion::ScreenMiddle => screen::resolve(buffer, ScreenPos::Middle, count1, state.viewport)?,
        Motion::ScreenBottom => screen::resolve(buffer, ScreenPos::Bottom, count1, state.viewport)?,
        Motion::SearchNext => search::repeat(buffer, position, count1, false, state)?,
        Motion::SearchPrev => search::repeat(buffer, position, count1, true, state)?,
        Motion::SearchWordForward => search::word(buffer, position, count1, true, state)?,
        Motion::SearchWordBackward => search::word(buffer, position, count1, false, state)?,
        Motion::Mark(name) => mark::resolve(buffer, name, false, state)?,
        Motion::MarkLine(name) => mark::resolve(buffer, name, true, state)?,
    };

    if motion.is_jump() {
        mark::record_jump(state, position);
    }

    match motion {
        Motion::Up | Motion::Down => {}
        Motion::LineEnd => state.desired = Some(Desired::End),
        _ => state.desired = None,
    }

    match motioncontext {
        MotionContext::Standalone => Ok(MotionResult::Cursor(cursor_after(buffer, &output))),
        MotionContext::WithOperator(operator) => {
            let target = build_target(buffer, position, &output)?;
            Ok(MotionResult::Operator(finalize_target(
                buffer, operator, target,
            )?))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::normal::{
        motion::{SearchSpec, Viewport},
        operator::Operator,
        target::MotionKind,
    };

    fn buffer_with_lines(lines: &[&str]) -> Buffer {
        Buffer {
            lines: lines.iter().map(|line| line.to_string()).collect(),
            row: 0,
            column: 0,
            modified: false,
            ending: crate::editor::buffer::LineEnding::Lf,
            final_newline: false
        }
    }

    fn at(line: usize, col: usize) -> Position {
        Position::new(line, col)
    }

    fn cursor_with(
        buffer: &Buffer,
        from: Position,
        motion: Motion,
        count: Option<usize>,
    ) -> Position {
        let mut state = MotionState::default();

        match resolve(
            buffer,
            from,
            motion,
            count,
            MotionContext::Standalone,
            &mut state,
        )
        .unwrap()
        {
            MotionResult::Cursor(position) => position,
            MotionResult::Operator(_) => panic!("expected cursor"),
        }
    }

    fn cursor(buffer: &Buffer, from: Position, motion: Motion) -> Position {
        cursor_with(buffer, from, motion, None)
    }

    fn cursor_count(buffer: &Buffer, from: Position, motion: Motion, count: usize) -> Position {
        cursor_with(buffer, from, motion, Some(count))
    }

    fn target(
        buffer: &Buffer,
        from: Position,
        motion: Motion,
        count: Option<usize>,
        operator: Operator,
    ) -> Target {
        let mut state = MotionState::default();
        let context = MotionContext::WithOperator(operator);

        match resolve(buffer, from, motion, count, context, &mut state).unwrap() {
            MotionResult::Operator(target) => target,
            MotionResult::Cursor(_) => panic!("expected target"),
        }
    }

    #[test]
    fn w_stops_on_empty_line() {
        let buffer = buffer_with_lines(&["foo", "", "bar"]);
        assert_eq!(cursor(&buffer, at(0, 0), Motion::Word), at(1, 0));
        assert_eq!(cursor(&buffer, at(1, 0), Motion::Word), at(2, 0));
    }

    #[test]
    fn e_skips_empty_line() {
        let buffer = buffer_with_lines(&["foo", "", "bar"]);
        assert_eq!(cursor(&buffer, at(0, 2), Motion::WordEnd), at(2, 2));
    }

    #[test]
    fn b_stops_on_empty_line() {
        let buffer = buffer_with_lines(&["foo", "", "bar"]);
        assert_eq!(cursor(&buffer, at(2, 0), Motion::WordBack), at(1, 0));
        assert_eq!(cursor(&buffer, at(1, 0), Motion::WordBack), at(0, 0));
    }

    #[test]
    fn accented_letters_are_word_characters() {
        let buffer = buffer_with_lines(&["café x"]);
        assert_eq!(cursor(&buffer, at(0, 0), Motion::Word), at(0, 5));
    }

    #[test]
    fn de_does_not_eat_following_space() {
        let buffer = buffer_with_lines(&["hello world"]);
        let result = target(&buffer, at(0, 0), Motion::WordEnd, None, Operator::Delete);
        assert_eq!(result, Target::chars(at(0, 0), at(0, 5)));
    }

    #[test]
    fn dw_on_last_word_keeps_newline() {
        let buffer = buffer_with_lines(&["foo bar", "baz"]);
        let result = target(&buffer, at(0, 4), Motion::Word, None, Operator::Delete);
        assert_eq!(result, Target::chars(at(0, 4), at(0, 7)));
    }

    #[test]
    fn cw_acts_like_ce() {
        let buffer = buffer_with_lines(&["foo bar"]);
        let result = target(&buffer, at(0, 0), Motion::Word, None, Operator::Change);
        assert_eq!(result, Target::chars(at(0, 0), at(0, 3)));
    }

    #[test]
    fn d5l_clamps_to_end_of_line() {
        let buffer = buffer_with_lines(&["abc"]);
        let result = target(&buffer, at(0, 1), Motion::Right, Some(5), Operator::Delete);
        assert_eq!(result, Target::chars(at(0, 1), at(0, 3)));
    }

    #[test]
    fn dj_is_linewise() {
        let buffer = buffer_with_lines(&["a", "b", "c"]);
        let result = target(&buffer, at(0, 0), Motion::Down, None, Operator::Delete);
        assert_eq!(result.kind, MotionKind::Line);
        assert_eq!(result, Target::lines(0, 1));
    }

    #[test]
    fn vertical_keeps_desired_column() {
        let buffer = buffer_with_lines(&["abcdef", "x", "abcdef"]);
        let mut state = MotionState::default();

        let first = resolve(
            &buffer,
            at(0, 4),
            Motion::Down,
            None,
            MotionContext::Standalone,
            &mut state,
        );
        let Ok(MotionResult::Cursor(first)) = first else {
            panic!()
        };
        assert_eq!(first, at(1, 0));

        let second = resolve(
            &buffer,
            first,
            Motion::Down,
            None,
            MotionContext::Standalone,
            &mut state,
        );
        let Ok(MotionResult::Cursor(second)) = second else {
            panic!()
        };
        assert_eq!(second, at(2, 4));
    }

    #[test]
    fn g_without_count_goes_to_last_line() {
        let buffer = buffer_with_lines(&["a", "b", "c"]);
        assert_eq!(cursor(&buffer, at(0, 0), Motion::FileEnd), at(2, 0));
    }

    #[test]
    fn g_with_count_goes_to_that_line() {
        let buffer = buffer_with_lines(&["a", "b", "c"]);
        assert_eq!(
            cursor_count(&buffer, at(2, 0), Motion::FileEnd, 1),
            at(0, 0)
        );
        assert_eq!(
            cursor_count(&buffer, at(0, 0), Motion::FileEnd, 99),
            at(2, 0)
        );
    }

    #[test]
    fn gg_goes_to_first_line() {
        let buffer = buffer_with_lines(&["a", "b", "c"]);
        assert_eq!(cursor(&buffer, at(2, 0), Motion::FileStart), at(0, 0));
        assert_eq!(
            cursor_count(&buffer, at(0, 0), Motion::FileStart, 2),
            at(1, 0)
        );
    }

    #[test]
    fn percent_goes_to_file_position() {
        let buffer = buffer_with_lines(&["a", "b", "c", "d"]);
        assert_eq!(
            cursor_count(&buffer, at(0, 0), Motion::MatchBracket, 50),
            at(1, 0)
        );
    }

    #[test]
    fn percent_over_100_fails() {
        let buffer = buffer_with_lines(&["a", "b"]);
        let mut state = MotionState::default();

        let result = resolve(
            &buffer,
            at(0, 0),
            Motion::MatchBracket,
            Some(101),
            MotionContext::Standalone,
            &mut state,
        );

        assert_eq!(result.err(), Some(ResolveError::Failed));
    }

    #[test]
    fn match_bracket_without_count() {
        let buffer = buffer_with_lines(&["(a [b] c)"]);
        assert_eq!(cursor(&buffer, at(0, 0), Motion::MatchBracket), at(0, 8));
        assert_eq!(cursor(&buffer, at(0, 1), Motion::MatchBracket), at(0, 5));
        assert_eq!(cursor(&buffer, at(0, 8), Motion::MatchBracket), at(0, 0));
    }

    #[test]
    fn d_percent_deletes_through_closing_bracket() {
        let buffer = buffer_with_lines(&["(a [b] c)"]);
        let result = target(
            &buffer,
            at(0, 0),
            Motion::MatchBracket,
            None,
            Operator::Delete,
        );
        assert_eq!(result, Target::chars(at(0, 0), at(0, 9)));
    }

    #[test]
    fn zero_count_is_invalid() {
        let buffer = buffer_with_lines(&["a"]);
        let mut state = MotionState::default();

        let result = resolve(
            &buffer,
            at(0, 0),
            Motion::Word,
            Some(0),
            MotionContext::Standalone,
            &mut state,
        );

        assert_eq!(result.err(), Some(ResolveError::InvalidCount));
    }

    fn step(
        buffer: &Buffer,
        state: &mut MotionState,
        from: Position,
        motion: Motion,
        count: Option<usize>,
    ) -> Option<Position> {
        match resolve(
            buffer,
            from,
            motion,
            count,
            MotionContext::Standalone,
            state,
        ) {
            Ok(MotionResult::Cursor(position)) => Some(position),
            _ => None,
        }
    }

    #[test]
    fn find_repeat_and_reverse() {
        let buffer = buffer_with_lines(&["a,b,c,d"]);
        let mut state = MotionState::default();

        let first = step(&buffer, &mut state, at(0, 0), Motion::FindChar(','), None);
        assert_eq!(first, Some(at(0, 1)));
        let second = step(&buffer, &mut state, at(0, 1), Motion::RepeatFind, None);
        assert_eq!(second, Some(at(0, 3)));
        let back = step(
            &buffer,
            &mut state,
            at(0, 3),
            Motion::RepeatFindReverse,
            None,
        );
        assert_eq!(back, Some(at(0, 1)));
    }

    #[test]
    fn repeat_without_find_fails() {
        let buffer = buffer_with_lines(&["a,b"]);
        let mut state = MotionState::default();

        assert_eq!(
            step(&buffer, &mut state, at(0, 0), Motion::RepeatFind, None),
            None
        );
    }

    #[test]
    fn till_repeat_skips_adjacent_match() {
        let buffer = buffer_with_lines(&["a,b,c,d"]);
        let mut state = MotionState::default();

        let first = step(&buffer, &mut state, at(0, 0), Motion::TillChar(','), None);
        assert_eq!(first, Some(at(0, 0)));
        let second = step(&buffer, &mut state, at(0, 0), Motion::RepeatFind, None);
        assert_eq!(second, Some(at(0, 2)));
    }

    #[test]
    fn double_backtick_toggles() {
        let buffer = buffer_with_lines(&["a", "b", "c", "d"]);
        let mut state = MotionState::default();

        let end = step(&buffer, &mut state, at(0, 0), Motion::FileEnd, None);
        assert_eq!(end, Some(at(3, 0)));
        let back = step(&buffer, &mut state, at(3, 0), Motion::Mark('`'), None);
        assert_eq!(back, Some(at(0, 0)));
        let again = step(&buffer, &mut state, at(0, 0), Motion::Mark('`'), None);
        assert_eq!(again, Some(at(3, 0)));
    }

    #[test]
    fn named_mark_roundtrip() {
        let buffer = buffer_with_lines(&["  a", "b"]);
        let mut state = MotionState::default();
        state.marks.insert('a', at(0, 2));

        assert_eq!(
            step(&buffer, &mut state, at(1, 0), Motion::Mark('a'), None),
            Some(at(0, 2))
        );
        assert_eq!(
            step(&buffer, &mut state, at(1, 0), Motion::MarkLine('a'), None),
            Some(at(0, 2))
        );
        assert_eq!(
            step(&buffer, &mut state, at(1, 0), Motion::Mark('z'), None),
            None
        );
    }

    #[test]
    fn search_forward_wraps() {
        let buffer = buffer_with_lines(&["foo bar", "baz foo"]);
        let mut state = MotionState::default();
        state.last_search = Some(SearchSpec {
            pattern: "foo".into(),
            forward: true,
        });

        let next = step(&buffer, &mut state, at(1, 4), Motion::SearchNext, None);
        assert_eq!(next, Some(at(0, 0)));
    }

    #[test]
    fn search_backward_and_repeat() {
        let buffer = buffer_with_lines(&["foo bar foo"]);
        let mut state = MotionState::default();
        state.last_search = Some(SearchSpec {
            pattern: "foo".into(),
            forward: false,
        });

        let first = step(&buffer, &mut state, at(0, 10), Motion::SearchNext, None);
        assert_eq!(first, Some(at(0, 8)));
        let second = step(&buffer, &mut state, at(0, 8), Motion::SearchNext, None);
        assert_eq!(second, Some(at(0, 0)));
    }

    #[test]
    fn star_matches_whole_word() {
        let buffer = buffer_with_lines(&["foo foobar foo"]);
        let mut state = MotionState::default();

        let hit = step(
            &buffer,
            &mut state,
            at(0, 0),
            Motion::SearchWordForward,
            None,
        );
        assert_eq!(hit, Some(at(0, 11)));
    }

    #[test]
    fn paragraph_motions() {
        let buffer = buffer_with_lines(&["a", "b", "", "c", "d"]);
        let mut state = MotionState::default();

        assert_eq!(
            step(
                &buffer,
                &mut state,
                at(0, 0),
                Motion::ParagraphForward,
                None
            ),
            Some(at(2, 0))
        );
        assert_eq!(
            step(
                &buffer,
                &mut state,
                at(2, 0),
                Motion::ParagraphForward,
                None
            ),
            Some(at(4, 0))
        );
        assert_eq!(
            step(
                &buffer,
                &mut state,
                at(4, 0),
                Motion::ParagraphForward,
                None
            ),
            None
        );
        assert_eq!(
            step(
                &buffer,
                &mut state,
                at(4, 0),
                Motion::ParagraphBackward,
                None
            ),
            Some(at(2, 0))
        );
        assert_eq!(
            step(
                &buffer,
                &mut state,
                at(2, 0),
                Motion::ParagraphBackward,
                None
            ),
            Some(at(0, 0))
        );
        assert_eq!(
            step(
                &buffer,
                &mut state,
                at(0, 0),
                Motion::ParagraphBackward,
                None
            ),
            None
        );
    }

    #[test]
    fn sentence_motions() {
        let buffer = buffer_with_lines(&["One. Two! Three"]);
        let mut state = MotionState::default();

        assert_eq!(
            step(&buffer, &mut state, at(0, 0), Motion::SentenceForward, None),
            Some(at(0, 5))
        );
        assert_eq!(
            step(&buffer, &mut state, at(0, 5), Motion::SentenceForward, None),
            Some(at(0, 10))
        );
        assert_eq!(
            step(
                &buffer,
                &mut state,
                at(0, 10),
                Motion::SentenceForward,
                None
            ),
            Some(at(0, 14))
        );
        assert_eq!(
            step(
                &buffer,
                &mut state,
                at(0, 14),
                Motion::SentenceForward,
                None
            ),
            None
        );
        assert_eq!(
            step(
                &buffer,
                &mut state,
                at(0, 10),
                Motion::SentenceBackward,
                None
            ),
            Some(at(0, 5))
        );
        assert_eq!(
            step(
                &buffer,
                &mut state,
                at(0, 0),
                Motion::SentenceBackward,
                None
            ),
            None
        );
    }

    #[test]
    fn screen_motions() {
        let buffer = buffer_with_lines(&["a", "b", "c", "d", "e", "f"]);
        let mut state = MotionState::default();
        state.viewport = Viewport { top: 1, height: 3 };

        assert_eq!(
            step(&buffer, &mut state, at(5, 0), Motion::ScreenTop, None),
            Some(at(1, 0))
        );
        assert_eq!(
            step(&buffer, &mut state, at(5, 0), Motion::ScreenMiddle, None),
            Some(at(2, 0))
        );
        assert_eq!(
            step(&buffer, &mut state, at(0, 0), Motion::ScreenBottom, None),
            Some(at(3, 0))
        );
    }

    #[test]
    fn next_and_prev_line() {
        let buffer = buffer_with_lines(&["a", "  b", "c"]);
        let mut state = MotionState::default();

        assert_eq!(
            step(
                &buffer,
                &mut state,
                at(0, 0),
                Motion::NextLineFirstNonBlank,
                None
            ),
            Some(at(1, 2))
        );
        assert_eq!(
            step(
                &buffer,
                &mut state,
                at(2, 0),
                Motion::NextLineFirstNonBlank,
                None
            ),
            None
        );
        assert_eq!(
            step(
                &buffer,
                &mut state,
                at(2, 0),
                Motion::PrevLineFirstNonBlank,
                None
            ),
            Some(at(1, 2))
        );
        assert_eq!(
            step(
                &buffer,
                &mut state,
                at(0, 0),
                Motion::PrevLineFirstNonBlank,
                None
            ),
            None
        );
        assert_eq!(
            step(
                &buffer,
                &mut state,
                at(0, 0),
                Motion::CurrentLineFirstNonBlank,
                Some(2)
            ),
            Some(at(1, 2))
        );
    }

    #[test]
    fn ge_goes_to_previous_word_end() {
        let buffer = buffer_with_lines(&["foo bar"]);
        let mut state = MotionState::default();

        assert_eq!(
            step(&buffer, &mut state, at(0, 4), Motion::WordEndBack, None),
            Some(at(0, 2))
        );
        assert_eq!(
            step(&buffer, &mut state, at(0, 0), Motion::WordEndBack, None),
            None
        );
    }

    #[test]
    fn huge_count_is_clamped() {
        let buffer = buffer_with_lines(&["abc", "def"]);
        let mut state = MotionState::default();

        assert_eq!(
            step(
                &buffer,
                &mut state,
                at(0, 0),
                Motion::Right,
                Some(usize::MAX / 2)
            ),
            Some(at(0, 2))
        );
        assert_eq!(
            step(&buffer, &mut state, at(0, 0), Motion::Down, Some(1000)),
            Some(at(1, 0))
        );
    }

    #[test]
    fn empty_buffer_line_is_stable() {
        let buffer = buffer_with_lines(&[""]);
        let mut state = MotionState::default();

        assert_eq!(
            step(&buffer, &mut state, at(0, 0), Motion::LineEnd, None),
            Some(at(0, 0))
        );
        assert_eq!(
            step(&buffer, &mut state, at(0, 0), Motion::Right, None),
            None
        );
        assert_eq!(
            step(&buffer, &mut state, at(0, 0), Motion::Down, None),
            None
        );
    }

    #[test]
    fn dd_style_delete_with_find() {
        let buffer = buffer_with_lines(&["a,b,c"]);
        let t = target(
            &buffer,
            at(0, 0),
            Motion::FindChar(','),
            None,
            Operator::Delete,
        );
        assert_eq!(t.kind, MotionKind::Char);
    }
}
