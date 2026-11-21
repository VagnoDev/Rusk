use crate::normal::parser::Key;

pub enum Action {
    InsertChar(char),
    SetNormal,
    SetCommand,
    EscCommand,
    SendCommand,
    InsertForward,
    InserBackward,
    ExplorerUp,
    ExplorerDown,
    EnterFile,
    SetExplorer,
    RemoveChar,
    InserNewLine,
    Quit,
    NormalKey(Key),
}
