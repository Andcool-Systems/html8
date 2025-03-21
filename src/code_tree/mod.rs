use crate::{
    definitions::start_def_check,
    libs::std::STD,
    math::math::{ExprToken, MathParser},
    parser::types::{ASTBody, ASTNode, PropType},
    types::typechecker::start_types_check,
};
use types::{
    ArgStruct, BlockStruct, BlockType, CallArgStruct, CallStruct, DataType, DefinitionType,
    FunctionDefinitionStruct, NodeType, VariableDefinitionStruct,
};

pub mod types;
use regex::Regex;

fn is_valid_identifier(s: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z_]+$").unwrap();
    re.is_match(s)
}

pub fn start_generating_code_tree(tree: ASTNode) -> NodeType {
    let mut tree = preprocess_code_tree(tree);

    match tree {
        NodeType::BLOCK(ref mut block_struct) if block_struct.tag == BlockType::Html => {
            for ref mut child in block_struct.children.iter_mut() {
                match child.as_mut() {
                    NodeType::BLOCK(ref mut block_struct)
                        if block_struct.tag == BlockType::Main =>
                    {
                        block_struct.children.splice(0..0, STD::use_lib());
                    }
                    NodeType::BLOCK(ref mut block_struct)
                        if block_struct.tag == BlockType::Head => {} // Will be used later
                    _ => panic!("Unexpected tag inside `html`!"),
                }
            }
        }
        _ => panic!("Unexpected root tag!"),
    }

    start_def_check(&mut tree);
    start_types_check(&mut tree);
    tree
}

#[derive(Debug)]
enum TempNodeType {
    Definition(DataType),
    Block(BlockType),
    Call,
}

fn get_data_type(str: String) -> Option<DataType> {
    match str {
        s if s == "int" => Some(DataType::Int),
        s if s == "str" => Some(DataType::Str),
        s if s == "bool" => Some(DataType::Bool),
        _ => None,
    }
}

fn preprocess_code_tree(tree: ASTNode) -> NodeType {
    let temp_node_type = match &tree.name {
        // Definitions
        s if s == "int" => TempNodeType::Definition(DataType::Int),
        s if s == "bool" => TempNodeType::Definition(DataType::Bool),
        s if s == "str" => TempNodeType::Definition(DataType::Str),
        s if s == "void" => TempNodeType::Definition(DataType::Void),

        // Blocks
        s if s == "html" => TempNodeType::Block(BlockType::Html),
        s if s == "head" => TempNodeType::Block(BlockType::Head),
        s if s == "main" => TempNodeType::Block(BlockType::Main),
        s if s == "div" => TempNodeType::Block(BlockType::Div),

        // Assign/Call
        _ if tree.self_closing => TempNodeType::Call,
        _ => todo!("Assign not yet implemented"),
    };

    let mut node_type = match temp_node_type {
        TempNodeType::Block(block_type) => NodeType::BLOCK(BlockStruct {
            tag: block_type,
            children: Vec::new(),
        }),
        TempNodeType::Definition(data_type) => {
            let definition_name = if let Some(prop) = tree.props.iter().find(|p| p.name == "name") {
                if let Some(PropType::Literal(new_name)) = &prop.value {
                    if is_valid_identifier(&new_name) {
                        new_name.to_string()
                    } else {
                        panic!("`{}` is not valid name!", new_name)
                    }
                } else {
                    panic!("Cannot use dynamic value for defining a variable name!")
                }
            } else {
                panic!("You should define name for variable!")
            };

            let is_func = tree.children.len() >= 1
                && tree
                    .children
                    .iter()
                    .all(|child| matches!(child, ASTBody::Tag(_)));
            if is_func {
                let args = tree
                    .props
                    .iter()
                    .filter(|prop| prop.name != "name")
                    .map(|prop| {
                        let data_type = prop.value.clone().unwrap_or_else(|| {
                            panic!("Function argument cannot be a flag: {}", prop.name)
                        });

                        if let PropType::Literal(v) = data_type {
                            return ArgStruct {
                                name: prop.name.clone(),
                                data_type: get_data_type(v).unwrap_or_else(|| {
                                    panic!(
                                        "Unknown data type for function argument: {:?}",
                                        prop.value
                                    )
                                }),
                            };
                        } else {
                            panic!(
                                "Function argument type cannot be a variable: {:?}",
                                prop.value
                            )
                        }
                    });
                NodeType::DEFINITION(DefinitionType::Function(FunctionDefinitionStruct {
                    data_type,
                    name: definition_name,
                    children: Vec::new(),
                    args: args.collect(),
                    must_be_compiled: true,
                }))
            } else if tree.children.len() == 1 {
                let value = match &tree.children[0] {
                    ASTBody::String(str) => str,
                    ASTBody::Tag(_) => {
                        todo!("Function calls inside variable definitions not yet implemented")
                    }
                };

                let is_const = tree
                    .props
                    .iter()
                    .find(|prop| prop.name == "const" && prop.value.is_none());

                if !tree
                    .props
                    .iter()
                    .filter(|prop| prop.name != "const" && prop.name != "name")
                    .collect::<Vec<_>>()
                    .is_empty()
                {
                    panic!("Function definition cannot take arguments")
                }

                let mut math = MathParser::new(value.to_string().chars());
                NodeType::DEFINITION(DefinitionType::Variable(VariableDefinitionStruct {
                    data_type,
                    name: definition_name,
                    value: math.parse_expr(),
                    is_const: is_const.is_some(),
                }))
            } else {
                panic!("All children should be a nodes, not values!");
            }
        }
        TempNodeType::Call => NodeType::CALL({
            let args = tree.props.iter().map(|prop| {
                let mut value = None;
                if let Some(val) = &prop.value {
                    value = match val {
                        PropType::Literal(s) => Some(ExprToken::Literal(s.to_string())),
                        PropType::Var(s) => Some(MathParser::new(s.chars()).parse_expr()),
                    };
                }

                CallArgStruct {
                    name: prop.name.clone(),
                    value,
                }
            });
            CallStruct {
                calling_name: tree.name,
                args: args.collect(),
            }
        }),
    };

    for child in tree.children {
        match child {
            ASTBody::Tag(node) => match node_type {
                NodeType::BLOCK(ref mut block_struct) => block_struct
                    .children
                    .push(Box::new(preprocess_code_tree(*node))),
                NodeType::DEFINITION(ref mut definition_type) => match definition_type {
                    DefinitionType::Function(fds) => {
                        fds.children.push(Box::new(preprocess_code_tree(*node)))
                    }
                    DefinitionType::Variable(_) => {
                        todo!("Function calls inside variable definitions not yet implemented")
                    }
                },
                NodeType::CALL(_) => unreachable!("HOW IT'S POSSIBLE??"),
                _ => todo!(),
            },
            ASTBody::String(s) => match node_type {
                NodeType::BLOCK(_) => panic!("String tags not supported inside blocks"),
                NodeType::DEFINITION(ref mut definition_type) => match definition_type {
                    DefinitionType::Function(_) => panic!("Cannot use string tags inside function"),
                    DefinitionType::Variable(ref mut fds) => {
                        let mut math = MathParser::new(s.chars());
                        fds.value = math.parse_expr();
                    }
                },
                NodeType::CALL(_) => unreachable!("HOW IT'S POSSIBLE??"),
                _ => todo!(),
            },
        }
    }

    node_type
}
