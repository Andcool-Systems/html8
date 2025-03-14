use crate::parser::types::{ASTNode, ASTProp};

#[derive(Debug, Clone)]
pub enum NodeType {
    BLOCK(BlockType),
    DEFINITION(DataTypes),
    CALL(String),
    ASSIGN(String),
}

#[derive(Debug, Clone)]
pub enum DataTypes {
    Int(Option<String>),
    Bool(Option<String>),
}

#[derive(Debug, Clone, Copy)]
pub enum BlockType {
    Html,
    Head,
    Main,
    Div,
}

impl NodeType {
    pub fn from(tag: &ASTNode) -> Self {
        match &tag.name {
            // Definitions
            s if s == "int" => Self::DEFINITION(DataTypes::Int(None)),
            s if s == "bool" => Self::DEFINITION(DataTypes::Bool(None)),

            // Blocks
            s if s == "html" => Self::BLOCK(BlockType::Html),
            s if s == "head" => Self::BLOCK(BlockType::Head),
            s if s == "main" => Self::BLOCK(BlockType::Main),
            s if s == "div" => Self::BLOCK(BlockType::Div),
            s if tag.self_closing => Self::CALL(s.to_string()),
            s => Self::ASSIGN(s.to_string()),
        }
    }
}

#[derive(Debug)]
pub enum Child {
    String(String),
    Tag(Box<Node>),
}

#[derive(Debug)]
pub enum FunctionVariable {
    Function,
    Variable,
}

#[derive(Debug)]
pub struct Node {
    pub node_type: NodeType,
    pub children: Vec<Child>,
    pub props: Vec<ASTProp>,
    pub func: Option<FunctionVariable>,
}
