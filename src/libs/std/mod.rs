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
    code_tree::types::{
        ArgStruct, CallArgStruct, CallStruct, DataType, DefinitionType, FunctionDefinitionStruct,
        NodeType,
    },
    compiler::CLang,
    math::{ExprToken, VariableType},
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

    fn compile_var_println(l: &VariableType) -> String {
        let format_key = match l.data_type {
            DataType::Int => String::from("%d"),
            DataType::Bool => String::from("%d"),
            DataType::Str => String::from("%s"),
            DataType::Void | DataType::Any => unreachable!(),
        };
        if !l.is_func {
            format!("printf(\"{}\\n\", {});", format_key, l.name)
        } else {
            format!("printf(\"<function at %d>\\n\", {});", l.name)
        }
    }

    pub fn compile_println(call: CallStruct) -> String {
        call.args
            .iter()
            .find(|a: &&CallArgStruct| a.name.eq("arg"))
            .map(|arg: &CallArgStruct| match &arg.value {
                Some(ExprToken::Literal(l)) => format!("printf(\"{}\\n\");", l),
                Some(ExprToken::Variable(l)) => Self::compile_var_println(l),
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
                Some(ExprToken::Variable(l)) => Self::compile_var_println(l),
                Some(_) => format!(
                    "printf(\"%d\", {});",
                    CLang::process_expr_token(arg.value.clone().unwrap())
                ),
                None => format!("printf(\"%d\", {});", true),
            };
        }
        String::new()
    }

    pub fn compile_return(call: CallStruct) -> String {
        if let Some(arg) = call.args.iter().find(|a: &&CallArgStruct| a.name.eq("arg")) {
            return match &arg.value {
                Some(ExprToken::Literal(l)) => format!("return \"{}\";", l),
                Some(ExprToken::Variable(l)) => format!("return {};", l.name),
                Some(_) => format!(
                    "return {};",
                    CLang::process_expr_token(arg.value.clone().unwrap())
                ),
                None => format!("return true;"),
            };
        }
        String::new()
    }

    pub fn compile_inc(call: CallStruct) -> String {
        if let Some(arg) = call.args.iter().find(|a: &&CallArgStruct| a.name.eq("arg")) {
            return match &arg.value {
                Some(ExprToken::Variable(l)) => format!("{}++;", l.name),
                _ => panic!("Cannot increment non-variable type"),
            };
        }
        String::new()
    }

    pub fn compile_dec(call: CallStruct) -> String {
        if let Some(arg) = call.args.iter().find(|a: &&CallArgStruct| a.name.eq("arg")) {
            return match &arg.value {
                Some(ExprToken::Variable(l)) => format!("{}--;", l.name),
                _ => panic!("Cannot increment non-variable type"),
            };
        }
        String::new()
    }
}
