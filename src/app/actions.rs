use crate::normal::parser::Key;

pub enum Action {
    InsertChar(char),
    SetNormal,
    EscCommand,
    SendCommand,
    ExplorerUp,
    ExplorerDown,
    EnterFile,
    RemoveChar,
    InserNewLine,
    NormalKey(Key),
}
