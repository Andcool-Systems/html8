use std::collections::HashMap;

use crate::code_tree::types::{ArgStruct, CallArgStruct};
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
                            data_type: arg.data_type,
                            name: arg.name.clone(),
                            value: ExprToken::Variable(arg.name.clone()),
                            is_const: true,
                        }),
                    );
                });
                
                fds.children
                    .iter_mut()
                    .for_each(|child: &mut Box<NodeType>| {
                        check(child, &mut scope);
                    });

                scope.insert(fds.name.clone(), Defined::Function(fds.clone()));
            }
            DefinitionType::Variable(ref mut vds) => {
                let value_type: DataType = vds.value.get_type(&scope);
                (vds.data_type != value_type).then(|| {
                    panic!(
                        "Value type for variable `{}` is incorrect! Expected `{:?}`, got `{:?}`",
                        vds.name, vds.data_type, value_type
                    );
                });
                vds.value.optimize(&scope);
                scope.insert(vds.name.clone(), Defined::Variable(vds.clone()));
            }
        },
        NodeType::CALL(ref mut call_struct) => {
            if let Some(Defined::Function(fds)) = scope.get(&call_struct.calling_name) {
                call_struct.args.iter_mut().for_each(|arg: &mut CallArgStruct| {
                    if let Some(ags) = fds.args.iter().find(|a: &&ArgStruct| a.name == arg.name) {
                        if let Some(argv) = arg.value.as_mut() {
                            let argv_type: DataType = argv.get_type(&scope);
                            argv.optimize(&scope);
                            
                            (ags.data_type != DataType::Any).then(|| {
                                if ags.data_type != argv_type {
                                    panic!(
                                        "Argument `{}` has wrong type! Expected: `{:?}`, got `{:?}`",
                                        ags.name, ags.data_type, argv_type
                                    );
                                }
                            });
                        }
                    }
                });
            }
        }
        NodeType::ASSIGN(_) => todo!(),
    }

    *defined = scope;
}
