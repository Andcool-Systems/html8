use std::collections::HashMap;

use crate::{
    code_tree::types::{DefinitionType, NodeType},
    definitions::Defined,
};

pub fn start_types_check(tree: &mut NodeType) {
    let mut defined: HashMap<String, Defined> = HashMap::new();
    check(tree, &mut defined);
}

fn check(tree: &mut NodeType, defined: &mut HashMap<String, Defined>) {
    let mut scope = defined.clone();

    match tree {
        NodeType::BLOCK(ref mut block_struct) => {
            for child in &mut block_struct.children {
                check(child, defined);
            }
        }
        NodeType::DEFINITION(ref mut definition_type) => match definition_type {
            DefinitionType::Function(fds) => {
                for child in &mut fds.children {
                    check(child, defined);
                }
                scope.insert(fds.name.clone(), Defined::Function(fds.clone()));
            }
            DefinitionType::Variable(ref mut vds) => {
                let value_type = vds.value.get_type(&scope);
                if vds.data_type != value_type {
                    panic!(
                        "Value type for variable `{}` is incorrect! Expected `{:?}`, got `{:?}`",
                        vds.name, vds.data_type, value_type
                    );
                }
                vds.value.optimize(&scope);
                scope.insert(vds.name.clone(), Defined::Variable(vds.clone()));
            }
        },
        NodeType::CALL(ref mut call_struct) => {
            if let Some(Defined::Function(fds)) = scope.get(&call_struct.calling_name) {
                for arg in &mut call_struct.args {
                    if let Some(v) = arg.value.as_mut() {
                        v.optimize(&scope);
                    }
                    if let (Some(afa), Some(ref mut argv)) = (
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
