/*
    HTML 8 Standard Library

    Definitions provider for functions:
    <println {} />
    <print {} />
    <return {} />
    <inc {} />
    <dec {} />
*/

use crate::{
    code_tree::types::CallArgStruct,
    code_tree::types::{
        ArgStruct, CallStruct, DataType, DefinitionType, FunctionDefinitionStruct, NodeType,
    },
    compiler::CLang,
    math::ExprToken,
};

pub struct Std;

impl Std {
    #[allow(clippy::vec_box)]
    pub fn use_lib() -> Vec<Box<NodeType>> {
        vec![
            Box::new(Self::build_println()),
            Box::new(Self::build_print()),
            Box::new(Self::build_return()),
            Box::new(Self::build_inc()),
            Box::new(Self::build_dec()),
        ]
    }

    fn build_println() -> NodeType {
        NodeType::DEFINITION(DefinitionType::Function(
            FunctionDefinitionStruct::new_internal(
                "println".to_string(),
                DataType::Void,
                vec![ArgStruct::new("arg".to_string(), DataType::Any)],
                false,
            ),
        ))
    }

    fn build_print() -> NodeType {
        NodeType::DEFINITION(DefinitionType::Function(
            FunctionDefinitionStruct::new_internal(
                "print".to_string(),
                DataType::Void,
                vec![ArgStruct::new("arg".to_string(), DataType::Any)],
                false,
            ),
        ))
    }

    fn build_return() -> NodeType {
        NodeType::DEFINITION(DefinitionType::Function(
            FunctionDefinitionStruct::new_internal(
                "return".to_string(),
                DataType::Void,
                vec![ArgStruct::new("arg".to_string(), DataType::Any)],
                false,
            ),
        ))
    }

    fn build_inc() -> NodeType {
        NodeType::DEFINITION(DefinitionType::Function(
            FunctionDefinitionStruct::new_internal(
                "inc".to_string(),
                DataType::Void,
                vec![ArgStruct::new("arg".to_string(), DataType::Any)],
                false,
            ),
        ))
    }

    fn build_dec() -> NodeType {
        NodeType::DEFINITION(DefinitionType::Function(
            FunctionDefinitionStruct::new_internal(
                "dec".to_string(),
                DataType::Void,
                vec![ArgStruct::new("arg".to_string(), DataType::Any)],
                false,
            ),
        ))
    }

    pub fn compile_println(call: CallStruct) -> String {
        call.args
            .iter()
            .find(|a: &&CallArgStruct| a.name.eq("arg"))
            .map(|arg: &CallArgStruct| match &arg.value {
                Some(ExprToken::Literal(l)) => format!("printf(\"{}\\n\");", l),
                Some(ExprToken::Variable(l)) => format!("printf(\"%s\\n\", {});", l),
                Some(_) => format!(
                    "printf(\"%d\\n\", {});",
                    CLang::process_expr_token(arg.value.clone().unwrap())
                ),
                None => format!("printf(\"%d\\n\", {});", true),
            })
            .unwrap_or_else(String::new)
    }

    pub fn compile_print(call: CallStruct) -> String {
        if let Some(arg) = call.args.iter().find(|a: &&CallArgStruct| a.name.eq("arg")) {
            return match &arg.value {
                Some(ExprToken::Literal(l)) => format!("printf(\"{}\");", l),
                Some(ExprToken::Variable(l)) => format!("printf(\"%s\", {});", l),
                Some(_) => format!(
                    "printf(\"%d\", {});",
                    CLang::process_expr_token(arg.value.clone().unwrap())
                ),
                None => format!("printf(\"%d\", {});", true),
            };
        }
        String::new()
    }
}
