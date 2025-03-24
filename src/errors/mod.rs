pub mod parser;
pub mod simple;

pub enum ErrorKind {
    Parsing,
    DefinitionCheck,
    TypeCheck,
}
