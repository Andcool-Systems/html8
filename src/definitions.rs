use std::collections::{HashMap, HashSet};

use crate::code_tree::types::{ArgStruct, AssignEnum, CallArgStruct, ServiceBlockType};
use crate::errors::simple::SimpleError;
use crate::errors::ErrorKind;
use crate::math::errors::DefinitionNotFound;
use crate::math::VariableType;
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

fn find_duplicate<T: Eq + std::hash::Hash + Clone>(arr: &[T]) -> Option<T> {
    let mut set = HashSet::new();
    for item in arr {
        if !set.insert(item) {
            return Some(item.clone());
        }
    }
    None
}

fn check_duplicate_def(args: Vec<String>, calling_name: String) {
    if let Some(duplicate) = find_duplicate(&args) {
        SimpleError::error(
            &format!(
                "Found duplicate argument `{}` in function: {}",
                duplicate, calling_name
            ),
            ErrorKind::DefinitionCheck,
        );
    }
}

fn check(tree: &mut NodeType, defined: &mut HashMap<String, Defined>) {
    match tree {
        NodeType::BLOCK(block_struct) => {
            let scope = defined.clone();
            block_struct
                .children
                .iter_mut()
                .for_each(|child: &mut Box<NodeType>| {
                    check(child, defined);
                });
            *defined = scope.clone();
        }
        NodeType::DEFINITION(definition_type) => match definition_type {
            DefinitionType::Function(fds) => {
                fds.args.clone().into_iter().for_each(|arg: ArgStruct| {
                    let var = Defined::Variable(VariableDefinitionStruct {
                        data_type: arg.data_type.clone(),
                        name: arg.name.clone(),
                        value: AssignEnum::Expr(ExprToken::Variable(VariableType::new(
                            arg.name.clone(),
                            arg.data_type,
                            false,
                        ))),
                        is_const: true,
                    });

                    defined.insert(arg.name.clone(), var);
                });

                check_duplicate_def(
                    fds.args.iter().map(|a| a.name.clone()).collect(),
                    fds.name.clone(),
                );

                let scope = defined.clone();
                fds.children
                    .iter_mut()
                    .for_each(|child: &mut Box<NodeType>| {
                        check(child, defined);
                    });

                *defined = scope.clone();

                defined.get(&fds.name).is_some().then(|| {
                    SimpleError::error(
                        &format!("Cannot redefine function `{}`", fds.name),
                        ErrorKind::DefinitionCheck,
                    );
                });

                defined.insert(fds.name.clone(), Defined::Function(fds.clone()));
            }
            DefinitionType::Variable(vds) => {
                match &vds.value {
                    AssignEnum::Expr(expr_token) => {
                        expr_token
                            .check_def(defined)
                            .unwrap_or_else(|e: DefinitionNotFound| {
                                SimpleError::error(
                                    &format!("Variable `{}` not defined", e.var_name),
                                    ErrorKind::DefinitionCheck,
                                )
                            });
                    }
                    AssignEnum::Call(node_type) => match *node_type.clone() {
                        NodeType::CALL(mut call_struct) => {
                            check_fn_call(defined, &mut call_struct);

                            defined.get(&call_struct.calling_name).unwrap_or_else(|| {
                                SimpleError::error(
                                    &format!("Function `{}` not defined", call_struct.calling_name),
                                    ErrorKind::DefinitionCheck,
                                )
                            });
                        }
                        _ => SimpleError::error(
                            &format!("Unexpected token inside `{}` definition", vds.name),
                            ErrorKind::DefinitionCheck,
                        ),
                    },
                    AssignEnum::None => unreachable!(),
                }

                defined.get(&vds.name).is_some().then(|| {
                    SimpleError::error(
                        &format!("Cannot redefine variable `{}`", vds.name),
                        ErrorKind::DefinitionCheck,
                    );
                });

                defined.insert(vds.name.clone(), Defined::Variable(vds.clone()));
            }
        },
        NodeType::CALL(call_struct) => check_fn_call(defined, call_struct),
        NodeType::ASSIGN(ref mut call_arg_struct) => {
            match defined.get(&call_arg_struct.name) {
                Some(Defined::Function(_)) => SimpleError::error(
                    &format!("Cannot assign value to `{}` function", call_arg_struct.name),
                    ErrorKind::DefinitionCheck,
                ),
                Some(Defined::Variable(v)) => {
                    if v.is_const {
                        SimpleError::error(
                            &format!(
                                "Cannot assign value to constant `{}` variable",
                                call_arg_struct.name
                            ),
                            ErrorKind::DefinitionCheck,
                        )
                    }
                }
                None => SimpleError::error(
                    &format!(
                        "Variable `{}` for assign not defined!",
                        call_arg_struct.name
                    ),
                    ErrorKind::DefinitionCheck,
                ),
            };

            match call_arg_struct.body.clone() {
                AssignEnum::Expr(expr_token) => {
                    expr_token
                        .check_def(defined)
                        .unwrap_or_else(|e: DefinitionNotFound| {
                            SimpleError::error(
                                &format!("Variable `{}` not defined", e.var_name),
                                ErrorKind::DefinitionCheck,
                            )
                        })
                }
                AssignEnum::Call(mut body) => match *body.clone() {
                    NodeType::CALL(_) => check(&mut body, defined),
                    _ => SimpleError::error(
                        &format!("Unexpected token inside `{}` assign", call_arg_struct.name),
                        ErrorKind::DefinitionCheck,
                    ),
                },
                AssignEnum::None => unreachable!(),
            }
        }
        NodeType::ServiceBlock(ref mut sbt) => match sbt {
            ServiceBlockType::For(for_struct) => {
                for_struct
                    .start
                    .check_def(&defined)
                    .unwrap_or_else(|e: DefinitionNotFound| {
                        SimpleError::error(
                            &format!("Variable `{}` not defined", e.var_name),
                            ErrorKind::DefinitionCheck,
                        )
                    });

                for_struct
                    .end
                    .check_def(&defined)
                    .unwrap_or_else(|e: DefinitionNotFound| {
                        SimpleError::error(
                            &format!("Variable `{}` not defined", e.var_name),
                            ErrorKind::DefinitionCheck,
                        )
                    });

                let scope = defined.clone();
                for_struct
                    .children
                    .iter_mut()
                    .for_each(|child: &mut Box<NodeType>| {
                        check(child, defined);
                    });
                *defined = scope.clone();
            }
        },
    }
}

fn check_fn_call(
    defined: &mut HashMap<String, Defined>,
    call_struct: &mut crate::code_tree::types::CallStruct,
) {
    let entry = defined.get(&call_struct.calling_name);

    entry.is_none().then(|| {
        SimpleError::error(
            &format!(
                "Cannot call undefined function: {}",
                call_struct.calling_name
            ),
            ErrorKind::DefinitionCheck,
        );
    });

    if let Some(Defined::Variable(vds)) = entry {
        SimpleError::error(
            &format!("Cannot call variable as function: {}", vds.name),
            ErrorKind::DefinitionCheck,
        );
    }

    if let Some(Defined::Function(f)) = entry {
        f.args.clone().into_iter().for_each(|arg: ArgStruct| {
            (!call_struct
                .args
                .iter()
                .any(|a: &CallArgStruct| a.name == arg.name))
            .then(|| {
                SimpleError::error(
                    &format!(
                        "Argument `{}` in function `{}` call is required",
                        arg.name, f.name
                    ),
                    ErrorKind::DefinitionCheck,
                );
            });
        });

        check_duplicate_def(
            call_struct.args.iter().map(|a| a.name.clone()).collect(),
            f.name.clone(),
        );
    }

    // Check call args
    call_struct.args.iter().for_each(|arg: &CallArgStruct| {
        if let Some(argv) = &arg.value {
            argv.check_def(defined)
                .unwrap_or_else(|e: DefinitionNotFound| {
                    SimpleError::error(
                        &format!("Variable `{}` not defined", e.var_name),
                        ErrorKind::DefinitionCheck,
                    )
                });
        }

        if let Some(Defined::Function(f)) = entry {
            (!f.args.iter().any(|a: &ArgStruct| a.name == arg.name)).then(|| {
                SimpleError::error(
                    &format!(
                        "Unexpected argument `{}` for function `{}`",
                        arg.name, f.name
                    ),
                    ErrorKind::DefinitionCheck,
                );
            });
        }
    });
}
