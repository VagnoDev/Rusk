use crate::{
    editor::buffer::Buffer,
    normal::{
        motion::Motion::{self, BigWordEndBack, WordEndBack},
        operator::Operator,
        resolver::{ResolveError, ResolveResult},
        scanner::{CharacterClass, Cursor, Step, WordMode},
        target::{Inclusivity, MotionOutput, Position},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Walk {
    pub pos: Position,
    pub ok: bool,
}

fn done(cursor: &Cursor, ok: bool) -> ResolveResult<Walk> {
    Ok(Walk {
        pos: cursor.pos(),
        ok,
    })
}

pub fn forward_word(
    buffer: &Buffer,
    start: Position,
    count: usize,
    wordmode: WordMode,
    stop_at_eol: bool,
) -> ResolveResult<Walk> {
    let mut cursor = Cursor::new(buffer, start)?;

    for remaining in (0..count).rev() {
        let is_last = remaining == 0;
        let start_class = cursor.class(wordmode);
        let on_last_line = cursor.on_last_line();
        let step = cursor.inc();

        if step == Step::Stuck || (step.crossed() && on_last_line) {
            return done(&cursor, false);
        }

        if step.crossed() && stop_at_eol && is_last {
            return done(&cursor, true);
        }

        if start_class != CharacterClass::Whitespace {
            while cursor.class(wordmode) == start_class {
                let step = cursor.inc();

                if step == Step::Stuck || (step.crossed() && stop_at_eol && is_last) {
                    return done(&cursor, true);
                }
            }
        }

        while cursor.class(wordmode) == CharacterClass::Whitespace {
            if cursor.at_empty_line() {
                break;
            }

            let step = cursor.inc();

            if step == Step::Stuck || (step.crossed() && on_last_line) {
                return done(&cursor, true);
            }
        }
    }

    done(&cursor, true)
}

pub fn word_end(
    buffer: &Buffer,
    start: Position,
    count: usize,
    wordmode: WordMode,
    stop: bool,
) -> ResolveResult<Walk> {
    let mut cursor = Cursor::new(buffer, start)?;
    let mut stop = stop;

    for _ in 0..count {
        let start_class = cursor.class(wordmode);

        if cursor.inc() == Step::Stuck {
            return done(&cursor, false);
        }

        if cursor.class(wordmode) == start_class && start_class != CharacterClass::Whitespace {
            if cursor.skip_class(start_class, wordmode, true) {
                return done(&cursor, false);
            }
        } else if !stop || start_class == CharacterClass::Whitespace {
            while cursor.class(wordmode) == CharacterClass::Whitespace {
                if cursor.inc() == Step::Stuck {
                    return done(&cursor, false);
                }
            }

            let class = cursor.class(wordmode);

            if cursor.skip_class(class, wordmode, true) {
                return done(&cursor, false);
            }
        }

        cursor.dec();
        stop = false;
    }

    done(&cursor, true)
}

pub fn word_back(
    buffer: &Buffer,
    start: Position,
    count: usize,
    wordmode: WordMode,
) -> ResolveResult<Walk> {
    let mut cursor = Cursor::new(buffer, start)?;

    for _ in 0..count {
        if cursor.dec() == Step::Stuck {
            return done(&cursor, false);
        }

        let mut on_empty_line = false;

        while cursor.class(wordmode) == CharacterClass::Whitespace {
            if cursor.at_empty_line() {
                on_empty_line = true;
                break;
            }

            if cursor.dec() == Step::Stuck {
                return done(&cursor, true);
            }
        }

        if on_empty_line {
            continue;
        }

        let class = cursor.class(wordmode);

        if cursor.skip_class(class, wordmode, false) {
            return done(&cursor, true);
        }

        cursor.inc();
    }

    done(&cursor, true)
}

pub fn word_end_back(
    buffer: &Buffer,
    start: Position,
    count: usize,
    wordmode: WordMode,
) -> ResolveResult<Walk> {
    let mut cursor = Cursor::new(buffer, start)?;

    for _ in 0..count {
        let class = cursor.class(wordmode);

        if class != CharacterClass::Whitespace {
            loop {
                if cursor.dec() == Step::Stuck {
                    return done(&cursor, false);
                }

                if cursor.class(wordmode) != class || cursor.at_empty_line() {
                    break;
                }
            }
        } else if cursor.dec() == Step::Stuck {
            return done(&cursor, false);
        }

        while cursor.class(wordmode) == CharacterClass::Whitespace && !cursor.at_empty_line() {
            if cursor.dec() == Step::Stuck {
                return done(&cursor, true);
            }
        }
    }

    done(&cursor, true)
}

fn wordmode_of(motion: Motion) -> Option<WordMode> {
    match motion {
        Motion::Word | Motion::WordBack | Motion::WordEnd | WordEndBack => Some(WordMode::Word),
        Motion::BigWord | Motion::BigWordBack | Motion::BigWordEnd | BigWordEndBack => {
            Some(WordMode::BigWord)
        }
        _ => None,
    }
}

pub fn resolve(
    buffer: &Buffer,
    position: Position,
    motion: Motion,
    count: usize,
    operator: Option<Operator>,
) -> ResolveResult<MotionOutput> {
    let wordmode = wordmode_of(motion).ok_or(ResolveError::Failed)?;

    let (walk, inclusivity) = match motion {
        Motion::Word | Motion::BigWord => {
            let on_word =
                Cursor::new(buffer, position)?.class(wordmode) != CharacterClass::Whitespace;

            if operator == Some(Operator::Change) && on_word {
                (
                    word_end(buffer, position, count, wordmode, true)?,
                    Inclusivity::Inclusive,
                )
            } else {
                (
                    forward_word(buffer, position, count, wordmode, operator.is_some())?,
                    Inclusivity::Exclusive,
                )
            }
        }
        Motion::WordBack | Motion::BigWordBack => (
            word_back(buffer, position, count, wordmode)?,
            Inclusivity::Exclusive,
        ),
        Motion::WordEndBack | Motion::BigWordEndBack => (
            word_end_back(buffer, position, count, wordmode)?,
            Inclusivity::Inclusive,
        ),
        _ => (
            word_end(buffer, position, count, wordmode, false)?,
            Inclusivity::Inclusive,
        ),
    };

    if !walk.ok && walk.pos == position {
        return Err(ResolveError::Failed);
    }

    Ok(MotionOutput::chars(walk.pos, inclusivity))
}
