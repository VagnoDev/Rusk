#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Quit,
    ForceQuit,
    Write(Option<String>),
    WriteQuit(Option<String>),
    Edit(Option<String>),
    ForceEdit(Option<String>),
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
        "q!" if path.is_none() => Command::ForceQuit,
        "w" => Command::Write(path),
        "wq" => Command::WriteQuit(path),
        "e" => Command::Edit(path),
        "e!" => Command::ForceEdit(path),
        _ => Command::Unknown,
    }
}
