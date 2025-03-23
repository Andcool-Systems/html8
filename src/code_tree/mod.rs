use crate::{
    definitions::start_def_check,
    libs::std::Std,
    math::{ExprToken, MathParser},
    parser::types::{ASTBody, ASTNode, PropType},
    types::typechecker::start_types_check,
};
use types::{
    ArgStruct, AssignEnum, AssignStruct, BlockStruct, BlockType, CallArgStruct, CallStruct,
    DataType, DefinitionType, ForStruct, FunctionDefinitionStruct, NodeType, ServiceBlockType,
    VariableDefinitionStruct,
};

pub mod types;
use crate::parser::types::ASTProp;
use regex::Regex;

fn is_valid_identifier(s: &str) -> bool {
    Regex::new(r"^[a-zA-Z_]+$").unwrap().is_match(s)
}

pub fn start_generating_code_tree(tree: ASTNode) -> NodeType {
    let mut tree: NodeType = preprocess_code_tree(tree);

    match tree {
        NodeType::BLOCK(ref mut block_struct) if block_struct.tag == BlockType::Html => {
            for ref mut child in block_struct.children.iter_mut() {
                match child.as_mut() {
                    NodeType::BLOCK(ref mut block_struct)
                        if block_struct.tag == BlockType::Main =>
                    {
                        block_struct.children.splice(0..0, Std::use_lib());
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

#[derive(Debug, Clone)]
enum TempNodeType {
    Definition(DataType),
    Block(BlockType),
    ServiceBlock(BlockType),
    Call,
    Assign,
}

fn get_data_type(str: String) -> Option<DataType> {
    match str {
        s if s == "int" => Some(DataType::Int),
        s if s == "str" => Some(DataType::Str),
        s if s == "bool" => Some(DataType::Bool),
        _ => None,
    }
}

fn generate_call_args(props: Vec<ASTProp>) -> Vec<CallArgStruct> {
    props
        .iter()
        .map(|prop: &ASTProp| {
            let value: Option<ExprToken> = prop
                .clone()
                .value
                .map(|val: PropType| match val {
                    PropType::Literal(s) => Some(ExprToken::Literal(s.to_string())),
                    PropType::Var(s) => Some(MathParser::new(s.chars()).parse_expr()),
                })
                .unwrap_or(None);

            CallArgStruct {
                name: prop.name.clone(),
                value,
            }
        })
        .collect()
}

fn preprocess_code_tree(tree: ASTNode) -> NodeType {
    let temp_node_type: TempNodeType = match &tree.name {
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
        s if s == "for" => TempNodeType::ServiceBlock(BlockType::For),

        // Assign/Call
        _ if tree.self_closing => TempNodeType::Call,
        _ => TempNodeType::Assign,
    };

    let mut node_type: NodeType = match temp_node_type.clone() {
        TempNodeType::ServiceBlock(block_type) => match block_type {
            BlockType::For => NodeType::ServiceBlock(ServiceBlockType::For({
                let args = generate_call_args(tree.props);

                let start = args
                    .iter()
                    .find(|a| a.name.eq("start"))
                    .expect("Argument `start` in for block is required")
                    .value
                    .clone()
                    .expect("Argument `start` cannot be bool");

                let end = args
                    .iter()
                    .find(|a| a.name.eq("end"))
                    .expect("Argument `end` in for block is required")
                    .value
                    .clone()
                    .expect("Argument `end` cannot be bool");

                let iter = args
                    .iter()
                    .find(|a| a.name.eq("i"))
                    .expect("Argument `i` in for block is required")
                    .value
                    .clone()
                    .expect("Argument `i` cannot be bool");

                let iter_name = match iter {
                    ExprToken::Literal(n) => n,
                    _ => panic!("Argument `i` must be a literal"),
                };

                let children = vec![Box::new(NodeType::DEFINITION(DefinitionType::Variable(
                    VariableDefinitionStruct {
                        data_type: DataType::Int,
                        name: iter_name.clone(),
                        value: AssignEnum::Expr(start.clone()),
                        is_const: false,
                    },
                )))];
                ForStruct {
                    start,
                    end,
                    iter_name,
                    children,
                }
            })),
            _ => unreachable!(),
        },
        TempNodeType::Block(block_type) => NodeType::BLOCK(BlockStruct {
            tag: block_type,
            children: Vec::new(),
        }),
        TempNodeType::Definition(data_type) => {
            let definition_name: String =
                if let Some(prop) = tree.props.iter().find(|p: &&ASTProp| p.name == "name") {
                    if let Some(PropType::Literal(new_name)) = &prop.value {
                        is_valid_identifier(new_name)
                            .then(|| new_name.to_string())
                            .unwrap_or_else(|| panic!("`{}` is not valid name!", new_name))
                    } else {
                        panic!("Cannot use dynamic value for defining a variable name!")
                    }
                } else {
                    panic!("You should define name for variable!")
                };

            let is_func: bool = (tree.children.len() > 1
                || matches!(tree.children.first(), Some(ASTBody::Tag(tag)) if tag.name.eq("return") && tag.self_closing))
                && tree
                    .children
                    .iter()
                    .all(|child: &ASTBody| matches!(child, ASTBody::Tag(_)));

            match is_func {
                true => {
                    let args = tree
                        .props
                        .iter()
                        .filter(|prop: &&ASTProp| prop.name != "name")
                        .map(|prop: &ASTProp| {
                            let data_type: PropType = prop.value.clone().unwrap_or_else(|| {
                                panic!("Function argument cannot be a flag: {}", prop.name)
                            });

                            matches!(data_type, PropType::Literal(_))
                                .then(|| {
                                    if let PropType::Literal(v) = data_type {
                                        ArgStruct {
                                            name: prop.name.clone(),
                                            data_type: get_data_type(v).unwrap_or_else(|| {
                                                panic!(
                                                    "Unknown data type for function argument: {:?}",
                                                    prop.value
                                                )
                                            }),
                                        }
                                    } else {
                                        unreachable!()
                                    }
                                })
                                .unwrap_or_else(|| {
                                    panic!(
                                        "Function argument type cannot be a variable: {:?}",
                                        prop.value
                                    )
                                })
                        });

                    NodeType::DEFINITION(DefinitionType::Function(FunctionDefinitionStruct {
                        data_type,
                        name: definition_name,
                        children: Vec::new(),
                        args: args.collect(),
                        must_be_compiled: true,
                    }))
                }
                false => match tree.children.len() {
                    1 => {
                        let value = match &tree.children[0] {
                            ASTBody::String(str) => AssignEnum::Expr(
                                MathParser::new(str.to_string().chars()).parse_expr(),
                            ),
                            ASTBody::Tag(tag) => {
                                AssignEnum::Call(Box::new(preprocess_code_tree(*tag.clone())))
                            }
                        };

                        let is_const: bool = tree
                            .props
                            .iter()
                            .any(|prop: &ASTProp| prop.name == "const" && prop.value.is_none());

                        (tree
                            .props
                            .iter()
                            .filter(|prop: &&ASTProp| prop.name != "const" && prop.name != "name")
                            .count()
                            == 0)
                            .then_some(())
                            .unwrap_or_else(|| {
                                panic!(
                                    "Variable `{}` definition cannot take arguments",
                                    definition_name
                                )
                            });

                        NodeType::DEFINITION(DefinitionType::Variable(VariableDefinitionStruct {
                            data_type,
                            name: definition_name,
                            value,
                            is_const,
                        }))
                    }
                    _ => panic!("All children should be nodes, not values!"),
                },
            }
        }
        TempNodeType::Call => NodeType::CALL(CallStruct {
            calling_name: tree.name,
            args: generate_call_args(tree.props),
        }),
        TempNodeType::Assign => NodeType::ASSIGN(AssignStruct::new(tree.name)),
    };

    tree.children
        .into_iter()
        .for_each(|child: ASTBody| match child {
            ASTBody::Tag(node) => match node_type {
                NodeType::BLOCK(ref mut block_struct) => block_struct
                    .children
                    .push(Box::new(preprocess_code_tree(*node))),
                NodeType::DEFINITION(ref mut definition_type) => match definition_type {
                    DefinitionType::Function(fds) => {
                        fds.children.push(Box::new(preprocess_code_tree(*node)))
                    }
                    DefinitionType::Variable(_) => {}
                },
                NodeType::ASSIGN(ref mut assign_struct) => {
                    assign_struct.body = AssignEnum::Call(Box::new(preprocess_code_tree(*node)))
                }
                NodeType::ServiceBlock(ref mut service_block_type) => match service_block_type {
                    ServiceBlockType::For(for_struct) => {
                        for_struct
                            .children
                            .push(Box::new(preprocess_code_tree(*node)));
                    }
                },
                _ => unreachable!(),
            },
            ASTBody::String(s) => match node_type {
                NodeType::BLOCK(_) | NodeType::ServiceBlock(_) => {
                    panic!("String tags not supported inside blocks")
                }
                NodeType::DEFINITION(ref mut definition_type) => match definition_type {
                    DefinitionType::Function(_) => panic!("Cannot use string tags inside function"),
                    DefinitionType::Variable(_) => {}
                },
                NodeType::ASSIGN(ref mut assign_struct) => {
                    assign_struct.body = AssignEnum::Expr(MathParser::new(s.chars()).parse_expr())
                }
                _ => unreachable!(),
            },
        });

    node_type
}
