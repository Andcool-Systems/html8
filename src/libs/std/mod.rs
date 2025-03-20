/*
    HTML 8 Standard Library

    Definitions provider for functions:
    <println {} />
    <return {} />
    <inc {} />
    <dec {} />
*/

use crate::code_tree::types::{
    ArgStruct, DataType, DefinitionType, FunctionDefinitionStruct, NodeType,
};

pub struct STD {}

impl STD {
    pub fn use_lib() -> Vec<Box<NodeType>> {
        vec![
            Box::new(Self::build_println()),
            Box::new(Self::build_return()),
            Box::new(Self::build_inc()),
            Box::new(Self::build_dec()),
        ]
    }

    fn build_println() -> NodeType {
        NodeType::DEFINITION(DefinitionType::Function(FunctionDefinitionStruct::new(
            "println".to_string(),
            DataType::Void,
            vec![ArgStruct::new("arg".to_string(), DataType::Any)],
        )))
    }

    fn build_return() -> NodeType {
        NodeType::DEFINITION(DefinitionType::Function(FunctionDefinitionStruct::new(
            "return".to_string(),
            DataType::Void,
            vec![ArgStruct::new("arg".to_string(), DataType::Any)],
        )))
    }

    fn build_inc() -> NodeType {
        NodeType::DEFINITION(DefinitionType::Function(FunctionDefinitionStruct::new(
            "inc".to_string(),
            DataType::Void,
            vec![ArgStruct::new("arg".to_string(), DataType::Any)],
        )))
    }

    fn build_dec() -> NodeType {
        NodeType::DEFINITION(DefinitionType::Function(FunctionDefinitionStruct::new(
            "dec".to_string(),
            DataType::Void,
            vec![ArgStruct::new("arg".to_string(), DataType::Any)],
        )))
    }
}
