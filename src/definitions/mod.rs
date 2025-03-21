use std::collections::HashMap;

use crate::{
    code_tree::types::{
        DefinitionType, FunctionDefinitionStruct, NodeType, VariableDefinitionStruct,
    },
    math::math::ExprToken,
};

#[derive(Debug, Clone)]
pub enum Defined {
    Variable(VariableDefinitionStruct),
    Function(FunctionDefinitionStruct),
}

pub fn start_def_check(tree: &mut NodeType) {
    let mut defined: HashMap<String, Defined> = HashMap::new();
    check(tree, &mut defined);
}

fn check(tree: &mut NodeType, defined: &mut HashMap<String, Defined>) {
    let scope = &mut defined.clone();

    match tree {
        NodeType::BLOCK(block_struct) => {
            for child in &mut block_struct.children {
                check(child, defined);
            }
        }
        NodeType::DEFINITION(definition_type) => match definition_type {
            DefinitionType::Function(fds) => {
                for arg in fds.args.clone() {
                    scope.insert(
                        arg.name.clone(),
                        Defined::Variable(VariableDefinitionStruct {
                            data_type: arg.data_type,
                            name: arg.name.clone(),
                            value: ExprToken::Variable(String::new()),
                            is_const: true,
                        }),
                    );
                }
                for child in &mut fds.children {
                    check(child, scope);
                }

                //*scope = fn_scope;

                if scope.get(&fds.name).is_some() {
                    panic!("Cannot redefine function `{}`", fds.name);
                }
                scope.insert(fds.name.clone(), Defined::Function(fds.clone()));
            }
            DefinitionType::Variable(vds) => {
                vds.value
                    .check_def(&scope)
                    .unwrap_or_else(|e| panic!("Variable `{}` not defined", e.var_name));

                if scope.get(&vds.name).is_some() {
                    panic!("Cannot redefine variable `{}`", vds.name);
                }
                scope.insert(vds.name.clone(), Defined::Variable(vds.clone()));
            }
        },
        NodeType::CALL(call_struct) => {
            // Get calling variable by name
            let entry = scope.get(&call_struct.calling_name);

            if entry.is_none() {
                panic!(
                    "Cannot call undefined function: {}",
                    call_struct.calling_name
                );
            }

            if let Some(Defined::Variable(vds)) = entry {
                panic!("Cannot call variable as function: {}", vds.name);
            }

            if let Some(Defined::Function(f)) = entry {
                for arg in f.args.clone() {
                    if !call_struct.args.iter().any(|a| a.name == arg.name) {
                        panic!(
                            "Argument `{}` in function `{}` call is required",
                            arg.name, f.name
                        );
                    }
                }
            }

            // Check call args
            for arg in &call_struct.args {
                if let Some(argv) = &arg.value {
                    argv.check_def(&scope)
                        .unwrap_or_else(|e| panic!("Variable `{}` not defined", e.var_name));
                }

                if let Some(Defined::Function(f)) = entry {
                    if !f.args.iter().any(|a| a.name == arg.name) {
                        panic!(
                            "Unexpected argument `{}` for function `{}`",
                            arg.name, f.name
                        );
                    }
                }
            }
        }
        NodeType::ASSIGN(_) => todo!(),
    }

    *defined = scope.clone();
}
