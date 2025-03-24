use std::collections::HashMap;

use crate::code_tree::types::{ArgStruct, AssignEnum, ServiceBlockType};
use crate::errors::simple::SimpleError;
use crate::errors::ErrorKind;
use crate::math::VariableType;
use crate::{
    code_tree::types::{DataType, DefinitionType, NodeType, VariableDefinitionStruct},
    definitions::Defined,
    math::ExprToken,
};

pub fn start_types_check(tree: &mut NodeType) {
    check(tree, &mut HashMap::<String, Defined>::new());
}

fn check(tree: &mut NodeType, defined: &mut HashMap<String, Defined>) {
    let mut scope: HashMap<String, Defined> = defined.clone();

    match tree {
        NodeType::BLOCK(ref mut block_struct) => {
            block_struct
                .children
                .iter_mut()
                .for_each(|child: &mut Box<NodeType>| {
                    check(child, defined);
                })
        }
        NodeType::DEFINITION(ref mut definition_type) => match definition_type {
            DefinitionType::Function(fds) => {
                fds.args.clone().into_iter().for_each(|arg: ArgStruct| {
                    scope.insert(
                        arg.name.clone(),
                        Defined::Variable(VariableDefinitionStruct {
                            data_type: arg.data_type.clone(),
                            name: arg.name.clone(),
                            value: AssignEnum::Expr(ExprToken::Variable(VariableType::new(
                                arg.name.clone(),
                                arg.data_type,
                                false,
                            ))),
                            is_const: true,
                        }),
                    );
                });

                fds.children
                    .iter_mut()
                    .for_each(|child: &mut Box<NodeType>| {
                        check(child, &mut scope);
                    });

                let return_node: Option<&Box<NodeType>> = fds.children.iter().find(|child| {
                    if let NodeType::CALL(ref call) = ***child {
                        call.calling_name == "return"
                    } else {
                        false
                    }
                });

                if !matches!(fds.data_type, DataType::Void) {
                    match return_node {
                        Some(return_node) => match *return_node.clone() {
                            NodeType::CALL(mut call_struct) => {
                                let return_value = call_struct
                                    .args
                                    .iter_mut()
                                    .find(|a| a.name == "arg")
                                    .unwrap();

                                let return_type = match &mut return_value.value {
                                    Some(expr_token) => expr_token.get_type(&scope),
                                    None => DataType::Bool,
                                };

                                if return_type != fds.data_type {
                                    SimpleError::error(
                                            &format!(
                                                "Return statement inside `{}` function has wrong type: Expected {:?}, got {:?}.",
                                                fds.name, fds.data_type, return_type
                                            ),
                                            ErrorKind::TypeCheck
                                        );
                                }
                            }
                            _ => SimpleError::error("Unknown return tag", ErrorKind::TypeCheck),
                        },
                        None => SimpleError::error(
                            &format!("Function `{}` must have return statement", fds.name),
                            ErrorKind::TypeCheck,
                        ),
                    }
                }

                scope.insert(fds.name.clone(), Defined::Function(fds.clone()));
            }
            DefinitionType::Variable(ref mut vds) => {
                let value_type = match &mut vds.value {
                    AssignEnum::Expr(ref mut expr_token) => {
                        expr_token.optimize(&scope);
                        expr_token.get_type(&scope)
                    }
                    AssignEnum::Call(node_type) => match *node_type.clone() {
                        NodeType::CALL(call_struct) => match scope.get(&call_struct.calling_name) {
                            Some(Defined::Function(f)) => f.data_type.clone(),
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    },
                    AssignEnum::None => unreachable!(),
                };
                (vds.data_type != value_type).then(|| {
                        SimpleError::error(&format!(
                            "Value type for variable `{}` is incorrect! Expected `{:?}`, got `{:?}`",
                            vds.name, vds.data_type, value_type
                        ), ErrorKind::TypeCheck);
                    });
                scope.insert(vds.name.clone(), Defined::Variable(vds.clone()));
            }
        },
        NodeType::CALL(ref mut call_struct) => {
            if let Some(Defined::Function(fds)) = scope.get(&call_struct.calling_name) {
                call_struct.args.iter_mut().for_each(|arg| {
                        if let Some(ags) = fds.args.iter().find(|a| a.name == arg.name) {
                            if let Some(argv) = arg.value.as_mut() {
                                let argv_type: DataType = argv.get_type(&scope);
                                argv.optimize(&scope);
                                (ags.data_type != DataType::Any).then(|| {
                                    if ags.data_type != argv_type {
                                        SimpleError::error(&format!(
                                            "Argument `{}` has wrong type! Expected: `{:?}`, got `{:?}`",
                                            ags.name, ags.data_type, argv_type
                                        ), ErrorKind::TypeCheck);
                                    }
                                });
                            }
                        }
                    });
            }
        }
        NodeType::ASSIGN(ref mut assign_struct) => match &mut assign_struct.body {
            AssignEnum::Expr(ref mut expr_token) => {
                if let Some(Defined::Variable(var)) = scope.get(&assign_struct.name) {
                    let expr_type = expr_token.get_type(&scope);
                    if var.data_type != expr_type {
                        SimpleError::error(
                            &format!(
                                "Assign to `{}` has wrong type! Expected: `{:?}`, got `{:?}`",
                                assign_struct.name, var.data_type, expr_type
                            ),
                            ErrorKind::TypeCheck,
                        );
                    }
                }
            }
            AssignEnum::Call(node_type) => match *node_type.clone() {
                NodeType::CALL(call_struct) => {
                    let call_type = scope.get(&call_struct.calling_name);
                    let assign_type = scope.get(&assign_struct.name);

                    if let (Some(Defined::Variable(var)), Some(Defined::Function(fun))) =
                        (assign_type, call_type)
                    {
                        if var.data_type != fun.data_type {
                            SimpleError::error(
                                &format!(
                                    "Assign to `{}` has wrong type! Expected: `{:?}`, got `{:?}`",
                                    assign_struct.name, var.data_type, fun.data_type
                                ),
                                ErrorKind::TypeCheck,
                            );
                        }
                    }
                }
                _ => unreachable!(),
            },
            AssignEnum::None => unreachable!(),
        },
        NodeType::ServiceBlock(sbt) => {
            match sbt {
                ServiceBlockType::For(for_struct) => {
                    for_struct.start
                    .get_type(&scope)
                    .ne(&DataType::Int)
                    .then(|| SimpleError::error(
                        &format!("Argument `start` inside for block has wrong type! Expected `Int`"),
                        ErrorKind::TypeCheck
                    ));
                    for_struct.end.get_type(&scope).ne(&DataType::Int).then(|| {
                        SimpleError::error(
                            &format!(
                                "Argument `end` inside for block has wrong type! Expected `Int`"
                            ),
                            ErrorKind::TypeCheck,
                        )
                    });

                    for_struct.start.optimize(&scope);
                    for_struct.end.optimize(&scope);

                    for_struct.children.iter_mut().for_each(|child| {
                        check(child, defined);
                    })
                }
            }
        }
    }

    *defined = scope;
}
