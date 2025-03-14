use crate::parser::types::{ASTBody, ASTNode, PropType};
use types::{Child, FunctionVariable, Node, NodeType};
mod types;
use regex::Regex;

fn is_valid_identifier(s: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z_]+$").unwrap();
    re.is_match(s)
}

pub fn process_code_tree(tree: ASTNode) -> Node {
    // Process current node
    let mut node = Node {
        node_type: NodeType::from(&tree),
        children: Vec::new(),
        props: tree.props,
        func: None,
    };

    if let NodeType::DEFINITION(data_type) = &mut node.node_type {
        if let Some(prop) = node.props.iter().find(|p| p.name == "name") {
            if let Some(PropType::Literal(new_name)) = &prop.value {
                if is_valid_identifier(&new_name) {
                    match data_type {
                        types::DataTypes::Int(name) | types::DataTypes::Bool(name) => {
                            *name = Some(new_name.to_string());
                        }
                    }
                } else {
                    panic!("`{}` is not valid name!", new_name)
                }
            } else {
                panic!("Cannot use dynamic value for defining a variable name!")
            }
        } else {
            panic!("You should define name for variable!")
        }
    }

    for child in tree.children {
        match child {
            ASTBody::Tag(ast_node) => node
                .children
                .push(Child::Tag(Box::new(process_code_tree(*ast_node)))),
            ASTBody::String(str) => node.children.push(Child::String(str)),
        }
    }

    // Check using combination of string / node in blocks
    let mut node_def_type: Option<FunctionVariable> = None;
    if node.children.len() > 1 {
        node_def_type = Some(FunctionVariable::Function);
        if !node.children.iter().all(|ch| match ch {
            Child::String(_) => false,
            Child::Tag(_) => true,
        }) {
            panic!("All children should be a nodes, not values!");
        }
    } else if node.children.len() != 0 {
        node_def_type = Some(FunctionVariable::Variable);
    }

    node.func = node_def_type;

    node
}
