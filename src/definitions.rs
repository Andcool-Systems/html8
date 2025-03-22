use std::collections::HashMap;

use crate::code_tree::types::{ArgStruct, CallArgStruct};
use crate::math::errors::DefinitionNotFound;
use crate::{
    code_tree::types::{
        DefinitionType, FunctionDefinitionStruct, NodeType, VariableDefinitionStruct,
    },
    math::ExprToken,
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
    let scope: &mut HashMap<String, Defined> = &mut defined.clone();

    match tree {
        NodeType::BLOCK(block_struct) => {
            block_struct
                .children
                .iter_mut()
                .for_each(|child: &mut Box<NodeType>| {
                    check(child, defined);
                });
        }
        NodeType::DEFINITION(definition_type) => match definition_type {
            DefinitionType::Function(fds) => {
                fds.args.clone().into_iter().for_each(|arg: ArgStruct| {
                    let var = Defined::Variable(VariableDefinitionStruct {
                        data_type: arg.data_type,
                        name: arg.name.clone(),
                        value: ExprToken::Variable(arg.name.clone()),
                        is_const: true,
                    });

                    scope.insert(arg.name.clone(), var);
                });

                fds.children
                    .iter_mut()
                    .for_each(|child: &mut Box<NodeType>| {
                        check(child, scope);
                    });

                //*scope = fn_scope;

                scope.get(&fds.name).is_some().then(|| {
                    panic!("Cannot redefine function `{}`", fds.name);
                });

                scope.insert(fds.name.clone(), Defined::Function(fds.clone()));
            }
            DefinitionType::Variable(vds) => {
                vds.value
                    .check_def(scope)
                    .unwrap_or_else(|e: DefinitionNotFound| {
                        panic!("Variable `{}` not defined", e.var_name)
                    });

                scope.get(&vds.name).is_some().then(|| {
                    panic!("Cannot redefine variable `{}`", vds.name);
                });

                scope.insert(vds.name.clone(), Defined::Variable(vds.clone()));
            }
        },
        NodeType::CALL(call_struct) => {
            // Get calling variable by name
            let entry = scope.get(&call_struct.calling_name);

            entry.is_none().then(|| {
                panic!(
                    "Cannot call undefined function: {}",
                    call_struct.calling_name
                );
            });

            if let Some(Defined::Variable(vds)) = entry {
                panic!("Cannot call variable as function: {}", vds.name);
            }

            if let Some(Defined::Function(f)) = entry {
                f.args.clone().into_iter().for_each(|arg: ArgStruct| {
                    (!call_struct
                        .args
                        .iter()
                        .any(|a: &CallArgStruct| a.name == arg.name))
                    .then(|| {
                        panic!(
                            "Argument `{}` in function `{}` call is required",
                            arg.name, f.name
                        );
                    });
                });
            }

            // Check call args
            call_struct.args.iter().for_each(|arg: &CallArgStruct| {
                if let Some(argv) = &arg.value {
                    argv.check_def(scope)
                        .unwrap_or_else(|e: DefinitionNotFound| {
                            panic!("Variable `{}` not defined", e.var_name)
                        });
                }

                if let Some(Defined::Function(f)) = entry {
                    (!f.args.iter().any(|a: &ArgStruct| a.name == arg.name)).then(|| {
                        panic!(
                            "Unexpected argument `{}` for function `{}`",
                            arg.name, f.name
                        );
                    });
                }
            });
        }
        NodeType::ASSIGN(_) => todo!(),
    }

    *defined = scope.clone();
}
