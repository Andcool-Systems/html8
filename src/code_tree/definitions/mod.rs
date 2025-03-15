use super::types::{DefinitionType, FunctionDefinitionStruct, NodeType};

#[derive(Debug, Clone)]
pub enum Defined {
    Variable(String),
    Function(FunctionDefinitionStruct),
}

pub fn start_def_check(tree: NodeType) {
    let mut defined: Vec<Defined> = Vec::new();
    check(tree, &mut defined);
}

fn search<'a>(defined: &'a Vec<Defined>, name: &str) -> Option<&'a Defined> {
    defined.iter().find(|d| match d {
        Defined::Variable(vn) => *vn == *name,
        Defined::Function(fds) => fds.name == name,
    })
}

pub fn check(tree: NodeType, defined: &mut Vec<Defined>) {
    let mut scope = defined.clone();

    match tree {
        NodeType::BLOCK(block_struct) => {
            for child in block_struct.children {
                check(*child, &mut scope);
            }
        }
        NodeType::DEFINITION(definition_type) => match definition_type {
            DefinitionType::Function(fds) => {
                let mut local_scope = scope.clone();
                for child in &fds.children {
                    check(*child.clone(), &mut local_scope);
                }
                scope.push(Defined::Function(fds));
            }
            DefinitionType::Variable(vds) => scope.push(Defined::Variable(vds.name)),
        },
        NodeType::CALL(call_struct) => {
            let entry = search(&scope, &call_struct.calling_name);

            if entry.is_none() {
                panic!(
                    "Cannot call undefined function: {}",
                    call_struct.calling_name
                );
            }

            if let Some(Defined::Variable(name)) = entry {
                panic!("Cannot call variable as function: {}", name);
            }

            // Check call args
            for arg in &call_struct.args {
                if arg.is_simple {
                    continue;
                }

                if search(&scope, &arg.value).is_none() {
                    panic!(
                        "Variable `{}` in arguments for call `{}` not defined",
                        arg.value, call_struct.calling_name
                    );
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
