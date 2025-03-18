use std::collections::HashMap;

use crate::{
    code_tree::types::{DefinitionType, NodeType},
    definitions::Defined,
};

pub fn start_types_check(tree: NodeType) {
    let mut defined: HashMap<String, Defined> = HashMap::new();
    check(tree, &mut defined);
}

fn check(tree: NodeType, defined: &mut HashMap<String, Defined>) {
    let mut scope = defined.clone();

    match tree {
        NodeType::BLOCK(block_struct) => {
            for child in block_struct.children {
                check(*child, &mut scope);
            }
        }
        NodeType::DEFINITION(definition_type) => {
            match definition_type {
                DefinitionType::Function(fds) => {
                    let mut local_scope = scope.clone();
                    for child in &fds.children {
                        check(*child.clone(), &mut local_scope);
                    }
                    scope.insert(fds.name.clone(), Defined::Function(fds));
                }
                DefinitionType::Variable(vds) => {
                    let value_type = vds.value.get_type(&scope);
                    if vds.data_type != value_type {
                        panic!("Value type for variable `{}` is incorrect! Expected `{:?}`, got `{:?}`", vds.name, vds.data_type, value_type);
                    }
                    scope.insert(vds.name.clone(), Defined::Variable(vds));
                }
            }
        }
        NodeType::CALL(call_struct) => {
            if let Some(Defined::Function(fds)) = scope.get(&call_struct.calling_name) {
                for arg in &call_struct.args {
                    if let (Some(afa), Some(argv)) = (
                        fds.args.iter().find(|a| a.name == arg.name),
                        arg.value.clone(),
                    ) {
                        let argv_type = argv.get_type(&scope);
                        if afa.data_type != argv_type {
                            panic!(
                                "Argument `{}` has wrong type! Expected: `{:?}`, got `{:?}`",
                                afa.name, afa.data_type, argv_type
                            );
                        }
                    }
                }
            }
        }
        NodeType::ASSIGN(_) => todo!(),
    }

    *defined = scope;
}
