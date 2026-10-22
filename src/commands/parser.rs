#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Quit,
    Write,
    WriteQuit,
    Unknown,
}

pub fn parse(input: &str) -> Command {
    let mut parts = input.split_whitespace();

    let Some(name) = parts.next() else {
        return Command::Unknown;
    };

    match name {
        "q" => Command::Quit,
        "w" => Command::Write,
        "wq" => Command::WriteQuit,
        _ => Command::Unknown,
    }
}