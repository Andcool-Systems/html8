use std::collections::HashMap;

use crate::{
    code_tree::types::{DataType, DefinitionType, NodeType, VariableDefinitionStruct},
    definitions::Defined, math::math::ExprToken,
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
                for arg in fds.args.clone() {
                    scope.insert(
                        arg.name.clone(),
                        Defined::Variable(VariableDefinitionStruct {
                            data_type: arg.data_type,
                            name: arg.name.clone(),
                            value: ExprToken::Variable(arg.name.clone()),
                            is_const: true,
                        }),
                    );
                }
                for child in &mut fds.children {
                    check(child, &mut scope);
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
                    if let (Some(ags), Some(argv)) = (
                        fds.args.iter().find(|a| a.name == arg.name),
                        arg.value.as_mut(),
                    ) {
                        let argv_type = argv.get_type(&scope);
                        argv.optimize(&scope);

                        if let DataType::Any = ags.data_type {
                            continue;
                        }

                        if ags.data_type != argv_type {
                            panic!(
                                "Argument `{}` has wrong type! Expected: `{:?}`, got `{:?}`",
                                ags.name, ags.data_type, argv_type
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
