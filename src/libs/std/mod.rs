/*
    HTML 8 Standard Library

    Definitions provider for functions:
    <println stdout="any" />
    <return val="any" />
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
        ]
    }

    fn build_println() -> NodeType {
        NodeType::DEFINITION(DefinitionType::Function(FunctionDefinitionStruct::new(
            "println".to_string(),
            DataType::Void,
            vec![ArgStruct::new("stdout".to_string(), DataType::Any)],
        )))
    }

    fn build_return() -> NodeType {
        NodeType::DEFINITION(DefinitionType::Function(FunctionDefinitionStruct::new(
            "return".to_string(),
            DataType::Void,
            vec![ArgStruct::new("val".to_string(), DataType::Any)],
        )))
    }
}
