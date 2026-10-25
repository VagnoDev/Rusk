#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Quit,
    Write(Option<String>),
    WriteQuit(Option<String>),
    Unknown,
}

pub fn parse(input: &str) -> Command {
    let mut parts = input.split_whitespace();

    let Some(name) = parts.next() else {
        return Command::Unknown;
    };

    let path = parts.next().map(String::from);

    match name {
        "q" if path.is_none() => Command::Quit,
        "w" => Command::Write(path),
        "wq" => Command::WriteQuit(path),
        _ => Command::Unknown,
    }
}