use std::collections::HashMap;

use crate::normal::{operator::Operator, target::Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Up,
    Down,
    Left,
    Right,
    Word,
    WordBack,
    WordEnd,
    WordEndBack,
    BigWord,
    BigWordBack,
    BigWordEnd,
    BigWordEndBack,
    LineStart,
    LineEnd,
    FirstNonBlank,
    LastNonBlank,
    NextLineFirstNonBlank,
    PrevLineFirstNonBlank,
    CurrentLineFirstNonBlank,
    ColumnGoto,
    FileStart,
    FileEnd,
    FindChar(char),
    FindCharBack(char),
    TillChar(char),
    TillCharBack(char),
    RepeatFind,
    RepeatFindReverse,
    MatchBracket,
    ParagraphForward,
    ParagraphBackward,
    SentenceForward,
    SentenceBackward,
    ScreenTop,
    ScreenMiddle,
    ScreenBottom,
    SearchNext,
    SearchPrev,
    SearchWordForward,
    SearchWordBackward,
    Mark(char),
    MarkLine(char),
}

impl Motion {
    pub fn is_jump(self) -> bool {
        matches!(
            self,
            Motion::FileStart
                | Motion::FileEnd
                | Motion::MatchBracket
                | Motion::ParagraphForward
                | Motion::ParagraphBackward
                | Motion::SentenceForward
                | Motion::SentenceBackward
                | Motion::ScreenTop
                | Motion::ScreenMiddle
                | Motion::ScreenBottom
                | Motion::SearchNext
                | Motion::SearchPrev
                | Motion::SearchWordForward
                | Motion::SearchWordBackward
                | Motion::Mark(_)
                | Motion::MarkLine(_)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionContext {
    Standalone,
    WithOperator(Operator),
}

impl MotionContext {
    pub fn operator(self) -> Option<Operator> {
        match self {
            MotionContext::Standalone => None,
            MotionContext::WithOperator(operator) => Some(operator),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FindSpec {
    pub ch: char,
    pub forward: bool,
    pub till: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Desired {
    Col(usize),
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchSpec {
    pub pattern: String,
    pub forward: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Viewport {
    pub top: usize,
    pub height: usize,
}

#[derive(Debug, Default)]
pub struct MotionState {
    pub last_find: Option<FindSpec>,
    pub desired: Option<Desired>,
    pub last_search: Option<SearchSpec>,
    pub marks: HashMap<char, Position>,
    pub viewport: Viewport,
}
