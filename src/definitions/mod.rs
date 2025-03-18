use std::collections::HashMap;

use crate::code_tree::types::{
    DefinitionType, FunctionDefinitionStruct, NodeType, VariableDefinitionStruct,
};

#[derive(Debug, Clone)]
pub enum Defined {
    Variable(VariableDefinitionStruct),
    Function(FunctionDefinitionStruct),
}

pub fn start_def_check(tree: NodeType) {
    let mut defined: HashMap<String, Defined> = HashMap::new();
    check(tree, &mut defined);
}

fn check(tree: NodeType, defined: &mut HashMap<String, Defined>) {
    let mut scope = defined.clone();

    match tree {
        NodeType::BLOCK(block_struct) => {
            block_struct
                .children
                .iter()
                .for_each(|c| check(*c.clone(), &mut scope));
        }
        NodeType::DEFINITION(definition_type) => match definition_type {
            DefinitionType::Function(fds) => {
                fds.children
                    .iter()
                    .for_each(|c| check(*c.clone(), &mut scope.clone()));
                scope.insert(fds.name.clone(), Defined::Function(fds));
            }
            DefinitionType::Variable(vds) => {
                vds.value
                    .check_def(&scope)
                    .unwrap_or_else(|e| panic!("Variable `{}` not defined", e.var_name));
                scope.insert(vds.name.clone(), Defined::Variable(vds));
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

    *defined = scope;
}
