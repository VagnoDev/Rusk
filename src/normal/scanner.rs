use crate::{
    editor::buffer::Buffer,
    normal::{
        resolver::{ResolveError, ResolveResult},
        target::Position,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterClass {
    Word,
    Punctuation,
    Whitespace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordMode {
    Word,
    BigWord,
}

pub fn is_blank(character: char) -> bool {
    character == ' ' || character == '\t'
}

pub fn is_keyword(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

pub fn scan(character: Option<char>, wordmode: WordMode) -> CharacterClass {
    match character {
        None => CharacterClass::Whitespace,
        Some(c) if is_blank(c) => CharacterClass::Whitespace,
        Some(_) if wordmode == WordMode::BigWord => CharacterClass::Word,
        Some(c) if is_keyword(c) => CharacterClass::Word,
        Some(_) => CharacterClass::Punctuation,
    }
}

pub fn line_len(buffer: &Buffer, line: usize) -> Option<usize> {
    buffer.line(line).map(|l| l.chars().count())
}

pub fn line_chars(buffer: &Buffer, line: usize) -> Option<Vec<char>> {
    buffer.line(line).map(|l| l.chars().collect())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Moved,
    OntoEol,
    LineChanged,
    Stuck,
}

impl Step {
    pub fn crossed(self) -> bool {
        matches!(self, Step::OntoEol | Step::LineChanged)
    }
}

pub struct Cursor<'a> {
    buffer: &'a Buffer,
    pos: Position,
    chars: Vec<char>,
}

impl<'a> Cursor<'a> {
    pub fn new(buffer: &'a Buffer, pos: Position) -> ResolveResult<Self> {
        let chars = line_chars(buffer, pos.line).ok_or(ResolveError::InvalidPosition)?;

        if pos.col > chars.len() {
            return Err(ResolveError::InvalidPosition);
        }

        Ok(Self { buffer, pos, chars })
    }

    pub fn pos(&self) -> Position {
        self.pos
    }

    pub fn current(&self) -> Option<char> {
        self.chars.get(self.pos.col).copied()
    }

    pub fn at_empty_line(&self) -> bool {
        self.pos.col == 0 && self.chars.is_empty()
    }

    pub fn on_last_line(&self) -> bool {
        self.pos.line + 1 >= self.buffer.line_count()
    }

    pub fn class(&self, wordmode: WordMode) -> CharacterClass {
        scan(self.current(), wordmode)
    }

    pub fn inc(&mut self) -> Step {
        if self.pos.col < self.chars.len() {
            self.pos.col += 1;

            if self.pos.col < self.chars.len() {
                Step::Moved
            } else {
                Step::OntoEol
            }
        } else if self.pos.line + 1 < self.buffer.line_count() {
            self.pos.line += 1;
            self.pos.col = 0;
            self.chars = line_chars(self.buffer, self.pos.line).unwrap_or_default();
            Step::LineChanged
        } else {
            Step::Stuck
        }
    }

    pub fn dec(&mut self) -> Step {
        if self.pos.col > 0 {
            self.pos.col -= 1;
            Step::Moved
        } else if self.pos.line > 0 {
            self.pos.line -= 1;
            self.chars = line_chars(self.buffer, self.pos.line).unwrap_or_default();
            self.pos.col = self.chars.len();
            Step::LineChanged
        } else {
            Step::Stuck
        }
    }

    pub fn skip_class(&mut self, class: CharacterClass, wordmode: WordMode, forward: bool) -> bool {
        while self.class(wordmode) == class {
            let step = if forward { self.inc() } else { self.dec() };

            if step == Step::Stuck {
                return true;
            }
        }

        false
    }
}
