use crate::normal::{
    motion::Motion,
    operator::{IndentDirection, Operator},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Ctrl(char),
    Enter,
    Esc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scroll {
    PageDown,
    PageUp,
    HalfDown,
    HalfUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Motion {
        count: Option<usize>,
        operator: Option<Operator>,
        direction: Option<IndentDirection>,
        motion: Motion,
    },
    Lines {
        count: Option<usize>,
        operator: Operator,
        direction: Option<IndentDirection>,
    },
    SetMark(char),
    Search {
        forward: bool,
    },
    Scroll {
        kind: Scroll,
        count: Option<usize>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parsed {
    Complete(Command),
    Incomplete,
    Invalid,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Awaiting {
    Nothing,
    G,
    Find,
    FindBack,
    Till,
    TillBack,
    Mark,
    MarkLine,
    SetMark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pending {
    key: char,
    operator: Operator,
    direction: Option<IndentDirection>,
}

#[derive(Debug)]
pub struct Parser {
    first: Option<usize>,
    second: Option<usize>,
    operator: Option<Pending>,
    awaiting: Awaiting,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

fn push_digit(slot: &mut Option<usize>, digit: usize) -> bool {
    match slot
        .unwrap_or(0)
        .checked_mul(10)
        .and_then(|n| n.checked_add(digit))
    {
        Some(n) => {
            *slot = Some(n);
            true
        }
        None => false,
    }
}

fn operator_of(c: char) -> Option<Pending> {
    let (operator, direction) = match c {
        'd' => (Operator::Delete, None),
        'y' => (Operator::Yank, None),
        'c' => (Operator::Change, None),
        '>' => (Operator::Indent, Some(IndentDirection::Increase)),
        '<' => (Operator::Indent, Some(IndentDirection::Decrease)),
        _ => return None,
    };

    Some(Pending {
        key: c,
        operator,
        direction,
    })
}

fn simple_motion(c: char) -> Option<Motion> {
    Some(match c {
        'h' => Motion::Left,
        'j' => Motion::Down,
        'k' => Motion::Up,
        'l' => Motion::Right,
        'w' => Motion::Word,
        'W' => Motion::BigWord,
        'b' => Motion::WordBack,
        'B' => Motion::BigWordBack,
        'e' => Motion::WordEnd,
        'E' => Motion::BigWordEnd,
        '0' => Motion::LineStart,
        '^' => Motion::FirstNonBlank,
        '$' => Motion::LineEnd,
        '|' => Motion::ColumnGoto,
        '+' => Motion::NextLineFirstNonBlank,
        '-' => Motion::PrevLineFirstNonBlank,
        '_' => Motion::CurrentLineFirstNonBlank,
        'G' => Motion::FileEnd,
        '%' => Motion::MatchBracket,
        '{' => Motion::ParagraphBackward,
        '}' => Motion::ParagraphForward,
        '(' => Motion::SentenceBackward,
        ')' => Motion::SentenceForward,
        'H' => Motion::ScreenTop,
        'M' => Motion::ScreenMiddle,
        'L' => Motion::ScreenBottom,
        ';' => Motion::RepeatFind,
        ',' => Motion::RepeatFindReverse,
        'n' => Motion::SearchNext,
        'N' => Motion::SearchPrev,
        '*' => Motion::SearchWordForward,
        '#' => Motion::SearchWordBackward,
        _ => return None,
    })
}

impl Parser {
    pub fn new() -> Self {
        Self {
            first: None,
            second: None,
            operator: None,
            awaiting: Awaiting::Nothing,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn is_idle(&self) -> bool {
        self.first.is_none()
            && self.second.is_none()
            && self.operator.is_none()
            && self.awaiting == Awaiting::Nothing
    }

    pub fn feed(&mut self, key: Key) -> Parsed {
        let result = self.step(key);

        match result {
            Parsed::Incomplete => {}
            _ => self.reset(),
        }

        result
    }

    fn count(&self) -> Option<Option<usize>> {
        match (self.first, self.second) {
            (Some(a), Some(b)) => a.checked_mul(b).map(Some),
            (Some(n), None) | (None, Some(n)) => Some(Some(n)),
            (None, None) => Some(None),
        }
    }

    fn finish(&self, motion: Motion) -> Parsed {
        let Some(count) = self.count() else {
            return Parsed::Invalid;
        };

        Parsed::Complete(Command::Motion {
            count,
            operator: self.operator.map(|p| p.operator),
            direction: self.operator.and_then(|p| p.direction),
            motion,
        })
    }

    fn shortcut(&self, operator: Operator, motion: Motion) -> Parsed {
        let Some(count) = self.count() else {
            return Parsed::Invalid;
        };

        Parsed::Complete(Command::Motion {
            count,
            operator: Some(operator),
            direction: None,
            motion,
        })
    }

    fn shortcut_lines(&self, operator: Operator) -> Parsed {
        let Some(count) = self.count() else {
            return Parsed::Invalid;
        };

        Parsed::Complete(Command::Lines {
            count,
            operator,
            direction: None,
        })
    }

    fn step(&mut self, key: Key) -> Parsed {
        if key == Key::Esc {
            return Parsed::Cancelled;
        }

        match self.awaiting {
            Awaiting::Nothing => {}
            Awaiting::G => return self.after_g(key),
            awaiting => return self.after_char(awaiting, key),
        }

        match key {
            Key::Enter => self.finish(Motion::NextLineFirstNonBlank),
            Key::Ctrl(c) => self.ctrl(c),
            Key::Char(c) => self.character(c),
            Key::Esc => Parsed::Cancelled,
        }
    }

    fn ctrl(&mut self, c: char) -> Parsed {
        if self.operator.is_some() {
            return Parsed::Invalid;
        }

        let kind = match c {
            'f' => Scroll::PageDown,
            'b' => Scroll::PageUp,
            'd' => Scroll::HalfDown,
            'u' => Scroll::HalfUp,
            _ => return Parsed::Invalid,
        };

        match self.count() {
            Some(count) => Parsed::Complete(Command::Scroll { kind, count }),
            None => Parsed::Invalid,
        }
    }

    fn after_g(&mut self, key: Key) -> Parsed {
        let Key::Char(c) = key else {
            return Parsed::Invalid;
        };

        let motion = match c {
            'g' => Motion::FileStart,
            'e' => Motion::WordEndBack,
            'E' => Motion::BigWordEndBack,
            '_' => Motion::LastNonBlank,
            _ => return Parsed::Invalid,
        };

        self.finish(motion)
    }

    fn after_char(&mut self, awaiting: Awaiting, key: Key) -> Parsed {
        let Key::Char(c) = key else {
            return Parsed::Invalid;
        };

        let motion = match awaiting {
            Awaiting::Find => Motion::FindChar(c),
            Awaiting::FindBack => Motion::FindCharBack(c),
            Awaiting::Till => Motion::TillChar(c),
            Awaiting::TillBack => Motion::TillCharBack(c),
            Awaiting::Mark => Motion::Mark(c),
            Awaiting::MarkLine => Motion::MarkLine(c),
            Awaiting::SetMark => return Parsed::Complete(Command::SetMark(c)),
            Awaiting::Nothing | Awaiting::G => return Parsed::Invalid,
        };

        self.finish(motion)
    }

    fn character(&mut self, c: char) -> Parsed {
        if let Some(digit) = c.to_digit(10) {
            let digit = digit as usize;
            let slot_started = if self.operator.is_some() {
                self.second.is_some()
            } else {
                self.first.is_some()
            };

            if digit != 0 || slot_started {
                let slot = if self.operator.is_some() {
                    &mut self.second
                } else {
                    &mut self.first
                };

                return if push_digit(slot, digit) {
                    Parsed::Incomplete
                } else {
                    Parsed::Invalid
                };
            }
        }

        if let Some(pending) = operator_of(c) {
            return match self.operator {
                None => {
                    self.operator = Some(pending);
                    Parsed::Incomplete
                }
                Some(current) if current.key == c => match self.count() {
                    Some(count) => Parsed::Complete(Command::Lines {
                        count,
                        operator: current.operator,
                        direction: current.direction,
                    }),
                    None => Parsed::Invalid,
                },
                Some(_) => Parsed::Invalid,
            };
        }

        match c {
            'g' => {
                self.awaiting = Awaiting::G;
                return Parsed::Incomplete;
            }
            'f' => return self.await_char(Awaiting::Find),
            'F' => return self.await_char(Awaiting::FindBack),
            't' => return self.await_char(Awaiting::Till),
            'T' => return self.await_char(Awaiting::TillBack),
            '`' => return self.await_char(Awaiting::Mark),
            '\'' => return self.await_char(Awaiting::MarkLine),
            _ => {}
        }

        if self.operator.is_none() {
            match c {
                'm' => return self.await_char(Awaiting::SetMark),
                '/' => return Parsed::Complete(Command::Search { forward: true }),
                '?' => return Parsed::Complete(Command::Search { forward: false }),
                'x' => return self.shortcut(Operator::Delete, Motion::Right),
                'X' => return self.shortcut(Operator::Delete, Motion::Left),
                'D' => return self.shortcut(Operator::Delete, Motion::LineEnd),
                'C' => return self.shortcut(Operator::Change, Motion::LineEnd),
                's' => return self.shortcut(Operator::Change, Motion::Right),
                'S' => return self.shortcut_lines(Operator::Change),
                'Y' => return self.shortcut_lines(Operator::Yank),
                _ => {}
            }
        }

        match simple_motion(c) {
            Some(motion) => self.finish(motion),
            None => Parsed::Invalid,
        }
    }

    fn await_char(&mut self, awaiting: Awaiting) -> Parsed {
        self.awaiting = awaiting;
        Parsed::Incomplete
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed_all(parser: &mut Parser, keys: &str) -> Parsed {
        let mut last = Parsed::Incomplete;
        for c in keys.chars() {
            last = parser.feed(Key::Char(c));
        }
        last
    }

    fn parse(keys: &str) -> Parsed {
        feed_all(&mut Parser::new(), keys)
    }

    fn motion(count: Option<usize>, operator: Option<Operator>, motion: Motion) -> Parsed {
        let direction = match operator {
            Some(Operator::Indent) => Some(IndentDirection::Increase),
            _ => None,
        };
        Parsed::Complete(Command::Motion {
            count,
            operator,
            direction,
            motion,
        })
    }

    fn plain(count: Option<usize>, operator: Option<Operator>, m: Motion) -> Parsed {
        Parsed::Complete(Command::Motion {
            count,
            operator,
            direction: None,
            motion: m,
        })
    }

    #[test]
    fn simple_motion_has_no_count() {
        assert_eq!(parse("w"), plain(None, None, Motion::Word));
    }

    #[test]
    fn count_is_parsed() {
        assert_eq!(parse("12j"), plain(Some(12), None, Motion::Down));
    }

    #[test]
    fn zero_alone_is_line_start() {
        assert_eq!(parse("0"), plain(None, None, Motion::LineStart));
    }

    #[test]
    fn zero_after_digit_extends_count() {
        assert_eq!(parse("10j"), plain(Some(10), None, Motion::Down));
    }

    #[test]
    fn zero_after_operator_is_line_start() {
        assert_eq!(
            parse("d0"),
            plain(None, Some(Operator::Delete), Motion::LineStart)
        );
    }

    #[test]
    fn zero_in_second_count_extends() {
        assert_eq!(
            parse("d20w"),
            plain(Some(20), Some(Operator::Delete), Motion::Word)
        );
    }

    #[test]
    fn counts_multiply() {
        assert_eq!(
            parse("2d3w"),
            plain(Some(6), Some(Operator::Delete), Motion::Word)
        );
    }

    #[test]
    fn single_second_count_is_used() {
        assert_eq!(
            parse("d3w"),
            plain(Some(3), Some(Operator::Delete), Motion::Word)
        );
    }

    #[test]
    fn count_overflow_is_invalid() {
        let mut parser = Parser::new();
        let results: Vec<Parsed> = "99999999999999999999999"
            .chars()
            .map(|c| parser.feed(Key::Char(c)))
            .collect();
        assert!(results.contains(&Parsed::Invalid));
    }

    #[test]
    fn multiplication_overflow_is_invalid() {
        let big = usize::MAX.to_string();
        let keys = format!("{big}d{big}w");
        assert_eq!(parse(&keys), Parsed::Invalid);
    }

    #[test]
    fn operator_then_motion() {
        assert_eq!(
            parse("dw"),
            plain(None, Some(Operator::Delete), Motion::Word)
        );
        assert_eq!(parse("yw"), plain(None, Some(Operator::Yank), Motion::Word));
        assert_eq!(
            parse("cw"),
            plain(None, Some(Operator::Change), Motion::Word)
        );
    }

    #[test]
    fn indent_operators_carry_direction() {
        assert_eq!(
            parse(">j"),
            motion(None, Some(Operator::Indent), Motion::Down)
        );
        assert_eq!(
            parse("<j"),
            Parsed::Complete(Command::Motion {
                count: None,
                operator: Some(Operator::Indent),
                direction: Some(IndentDirection::Decrease),
                motion: Motion::Down,
            })
        );
    }

    #[test]
    fn doubled_operators_are_lines() {
        for (keys, operator) in [
            ("dd", Operator::Delete),
            ("yy", Operator::Yank),
            ("cc", Operator::Change),
            (">>", Operator::Indent),
            ("<<", Operator::Indent),
        ] {
            assert!(matches!(
                parse(keys),
                Parsed::Complete(Command::Lines { count: None, operator: o, .. }) if o == operator
            ));
        }
    }

    #[test]
    fn doubled_operator_counts_multiply() {
        assert!(matches!(
            parse("2d3d"),
            Parsed::Complete(Command::Lines {
                count: Some(6),
                operator: Operator::Delete,
                ..
            })
        ));
        assert!(matches!(
            parse("3dd"),
            Parsed::Complete(Command::Lines { count: Some(3), .. })
        ));
    }

    #[test]
    fn mismatched_operators_are_invalid() {
        assert_eq!(parse("dy"), Parsed::Invalid);
        assert_eq!(parse("><"), Parsed::Invalid);
    }

    #[test]
    fn g_prefix() {
        assert_eq!(parse("gg"), plain(None, None, Motion::FileStart));
        assert_eq!(parse("ge"), plain(None, None, Motion::WordEndBack));
        assert_eq!(parse("gE"), plain(None, None, Motion::BigWordEndBack));
        assert_eq!(parse("g_"), plain(None, None, Motion::LastNonBlank));
        assert_eq!(parse("gx"), Parsed::Invalid);
    }

    #[test]
    fn g_prefix_with_operator_and_count() {
        assert_eq!(
            parse("d3gg"),
            plain(Some(3), Some(Operator::Delete), Motion::FileStart)
        );
    }

    #[test]
    fn find_motions_take_a_char() {
        assert_eq!(parse("fx"), plain(None, None, Motion::FindChar('x')));
        assert_eq!(parse("Fx"), plain(None, None, Motion::FindCharBack('x')));
        assert_eq!(parse("tx"), plain(None, None, Motion::TillChar('x')));
        assert_eq!(parse("Tx"), plain(None, None, Motion::TillCharBack('x')));
        assert_eq!(parse("2fx"), plain(Some(2), None, Motion::FindChar('x')));
        assert_eq!(
            parse("dfx"),
            plain(None, Some(Operator::Delete), Motion::FindChar('x'))
        );
    }

    #[test]
    fn find_accepts_digit_and_zero_chars() {
        assert_eq!(parse("f0"), plain(None, None, Motion::FindChar('0')));
        assert_eq!(parse("f3"), plain(None, None, Motion::FindChar('3')));
    }

    #[test]
    fn marks() {
        assert_eq!(parse("`a"), plain(None, None, Motion::Mark('a')));
        assert_eq!(parse("'a"), plain(None, None, Motion::MarkLine('a')));
        assert_eq!(parse("``"), plain(None, None, Motion::Mark('`')));
        assert_eq!(parse("ma"), Parsed::Complete(Command::SetMark('a')));
    }

    #[test]
    fn invalid_key_after_operator_cancels() {
        assert_eq!(parse("dm"), Parsed::Invalid);
        assert_eq!(parse("d/"), Parsed::Invalid);
        assert_eq!(parse("dx"), Parsed::Invalid);
    }

    #[test]
    fn parser_resets_after_invalid() {
        let mut parser = Parser::new();
        assert_eq!(feed_all(&mut parser, "dm"), Parsed::Invalid);
        assert!(parser.is_idle());
        assert_eq!(feed_all(&mut parser, "w"), plain(None, None, Motion::Word));
    }

    #[test]
    fn escape_cancels_everything() {
        let mut parser = Parser::new();
        assert_eq!(feed_all(&mut parser, "2d"), Parsed::Incomplete);
        assert_eq!(parser.feed(Key::Esc), Parsed::Cancelled);
        assert!(parser.is_idle());
        assert_eq!(
            feed_all(&mut parser, "gg"),
            plain(None, None, Motion::FileStart)
        );
    }

    #[test]
    fn escape_cancels_awaiting_char() {
        let mut parser = Parser::new();
        assert_eq!(feed_all(&mut parser, "f"), Parsed::Incomplete);
        assert_eq!(parser.feed(Key::Esc), Parsed::Cancelled);
        assert!(parser.is_idle());
    }

    #[test]
    fn shortcuts() {
        assert_eq!(
            parse("x"),
            plain(None, Some(Operator::Delete), Motion::Right)
        );
        assert_eq!(
            parse("X"),
            plain(None, Some(Operator::Delete), Motion::Left)
        );
        assert_eq!(
            parse("D"),
            plain(None, Some(Operator::Delete), Motion::LineEnd)
        );
        assert_eq!(
            parse("C"),
            plain(None, Some(Operator::Change), Motion::LineEnd)
        );
        assert_eq!(
            parse("s"),
            plain(None, Some(Operator::Change), Motion::Right)
        );
        assert_eq!(
            parse("3x"),
            plain(Some(3), Some(Operator::Delete), Motion::Right)
        );
        assert!(matches!(
            parse("S"),
            Parsed::Complete(Command::Lines {
                operator: Operator::Change,
                count: None,
                ..
            })
        ));
        assert!(matches!(
            parse("2Y"),
            Parsed::Complete(Command::Lines {
                operator: Operator::Yank,
                count: Some(2),
                ..
            })
        ));
    }

    #[test]
    fn enter_is_next_line_first_non_blank() {
        let mut parser = Parser::new();
        assert_eq!(
            parser.feed(Key::Enter),
            plain(None, None, Motion::NextLineFirstNonBlank)
        );
        assert_eq!(feed_all(&mut parser, "d"), Parsed::Incomplete);
        assert_eq!(
            parser.feed(Key::Enter),
            plain(None, Some(Operator::Delete), Motion::NextLineFirstNonBlank)
        );
    }

    #[test]
    fn search_commands() {
        assert_eq!(
            parse("/"),
            Parsed::Complete(Command::Search { forward: true })
        );
        assert_eq!(
            parse("?"),
            Parsed::Complete(Command::Search { forward: false })
        );
    }

    #[test]
    fn scroll_commands() {
        let mut parser = Parser::new();
        for (c, kind) in [
            ('f', Scroll::PageDown),
            ('b', Scroll::PageUp),
            ('d', Scroll::HalfDown),
            ('u', Scroll::HalfUp),
        ] {
            assert_eq!(
                parser.feed(Key::Ctrl(c)),
                Parsed::Complete(Command::Scroll { kind, count: None })
            );
        }
        assert_eq!(parser.feed(Key::Ctrl('x')), Parsed::Invalid);
    }

    #[test]
    fn scroll_is_invalid_after_operator() {
        let mut parser = Parser::new();
        feed_all(&mut parser, "d");
        assert_eq!(parser.feed(Key::Ctrl('f')), Parsed::Invalid);
    }

    #[test]
    fn every_simple_motion_key() {
        let table = [
            ('h', Motion::Left),
            ('j', Motion::Down),
            ('k', Motion::Up),
            ('l', Motion::Right),
            ('w', Motion::Word),
            ('W', Motion::BigWord),
            ('b', Motion::WordBack),
            ('B', Motion::BigWordBack),
            ('e', Motion::WordEnd),
            ('E', Motion::BigWordEnd),
            ('^', Motion::FirstNonBlank),
            ('$', Motion::LineEnd),
            ('|', Motion::ColumnGoto),
            ('+', Motion::NextLineFirstNonBlank),
            ('-', Motion::PrevLineFirstNonBlank),
            ('_', Motion::CurrentLineFirstNonBlank),
            ('G', Motion::FileEnd),
            ('%', Motion::MatchBracket),
            ('{', Motion::ParagraphBackward),
            ('}', Motion::ParagraphForward),
            ('(', Motion::SentenceBackward),
            (')', Motion::SentenceForward),
            ('H', Motion::ScreenTop),
            ('M', Motion::ScreenMiddle),
            ('L', Motion::ScreenBottom),
            (';', Motion::RepeatFind),
            (',', Motion::RepeatFindReverse),
            ('n', Motion::SearchNext),
            ('N', Motion::SearchPrev),
            ('*', Motion::SearchWordForward),
            ('#', Motion::SearchWordBackward),
        ];
        for (c, m) in table {
            assert_eq!(parse(&c.to_string()), plain(None, None, m), "key {c}");
        }
    }

    #[test]
    fn unknown_key_is_invalid() {
        assert_eq!(parse("Z"), Parsed::Invalid);
        assert_eq!(parse("q"), Parsed::Invalid);
    }
}
