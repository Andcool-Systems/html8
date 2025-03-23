pub mod types;

use compiler_core::iter::Iter;
use types::{ASTBody, ASTNode, ASTProp, PropType};

#[derive(Debug)]
enum ParseState {
    None,
    Tag,
    Props,
    Body,
    ClosingTag,
}

pub fn start_parse(contents: String) -> ASTNode {
    let mut iter: Iter<char> = Iter::from(contents.chars());
    parse(&mut iter)
}

pub fn parse(iter: &mut Iter<char>) -> ASTNode {
    let mut parse_state = ParseState::None;
    let mut tag = ASTNode {
        self_closing: false,
        name: String::new(),
        children: Vec::new(),
        props: Vec::new(),
    };

    let (mut buffer, mut closing_tag): (String, String) = (String::new(), String::new());
    while let Some(char) = iter.next() {
        match char {
            '<' => match iter.peek() {
                Some('!') => handle_comment(iter),
                Some(_) => match parse_state {
                    ParseState::None => parse_state = ParseState::Tag,
                    ParseState::Body => {
                        buffer = buffer.trim().to_owned();

                        (!buffer.is_empty()).then(|| {
                            tag.children.push(ASTBody::String(buffer.clone()));
                            buffer.clear();
                        });

                        if let Some('/') = iter.peek() {
                            iter.next(); // Consume '/'
                            parse_state = ParseState::ClosingTag;
                        } else {
                            iter.step_back();
                            tag.children.push(ASTBody::Tag(Box::new(parse(iter))));
                        }
                    }
                    _ => panic!("Unexpected `<` tag"),
                },
                None => panic!("Unexpected EOF"),
            },
            '>' => match parse_state {
                ParseState::Props | ParseState::Tag => parse_state = ParseState::Body,
                ParseState::ClosingTag => {
                    (tag.name != closing_tag).then(|| {
                        panic!(
                            "Unexpected closing tag: </{}>. Expected </{}>",
                            closing_tag, tag.name
                        );
                    });
                    return tag;
                }
                _ => panic!("Unexpected `>` tag"),
            },
            '/' => {
                if let Some('>') = iter.peek() {
                    tag.self_closing = true;
                    iter.next();
                    return tag;
                }
            }
            char if !char.is_whitespace() => match parse_state {
                ParseState::Tag => tag.name.push(char),
                ParseState::Props => tag.props.push(process_prop(iter)),
                ParseState::ClosingTag => closing_tag.push(char),
                ParseState::Body => buffer.push(char),
                _ => panic!("Unexpected literal"),
            },
            char if char.is_whitespace() => match parse_state {
                ParseState::Tag => parse_state = ParseState::Props,
                ParseState::Body => buffer.push(char),
                _ => {}
            },
            _ => {}
        }
    }

    panic!("Unexpected EOF");
}

fn handle_comment(iter: &mut Iter<char>) {
    iter.step_back();

    if iter.next() != Some('<')
        || iter.next() != Some('!')
        || iter.next() != Some('-')
        || iter.next() != Some('-')
    {
        return;
    }

    while let Some(c) = iter.next() {
        if c == '-' && iter.peek() == Some('-') {
            iter.next();
            if iter.peek() == Some('>') {
                iter.next();
                break;
            }
        }
    }
}

#[derive(Debug)]
enum PropParseState {
    Name,
    Value,
    Eq,
}

enum PropValueType {
    None,
    Literal,
    Var,
}

fn process_prop(iter: &mut Iter<char>) -> ASTProp {
    iter.step_back();
    let mut parse_state = PropParseState::Name;
    let mut prop = ASTProp {
        name: String::new(),
        value: None,
    };

    let mut buffer: String = String::new();
    let mut value_type = PropValueType::None;
    while let Some(char) = iter.peek() {
        match char {
            '=' => match parse_state {
                PropParseState::Name => {
                    parse_state = PropParseState::Eq;
                    iter.next();
                }
                _ => panic!("Unexpected `=`"),
            },
            '"' => match parse_state {
                PropParseState::Eq => {
                    value_type = PropValueType::Literal;
                    parse_state = PropParseState::Value;
                    iter.next();
                }
                PropParseState::Value => match value_type {
                    PropValueType::Literal => {
                        parse_state = PropParseState::Name;
                        iter.next();
                    }
                    _ => buffer.push(iter.next().unwrap()),
                },
                PropParseState::Name if prop.name.is_empty() => {
                    prop.name = "arg".to_string();
                    value_type = PropValueType::Literal;
                    parse_state = PropParseState::Value;
                    iter.next();
                }
                _ => panic!("Unexpected `\"`"),
            },
            '{' => match parse_state {
                PropParseState::Eq => {
                    value_type = PropValueType::Var;
                    parse_state = PropParseState::Value;
                    iter.next();
                }
                PropParseState::Name if prop.name.is_empty() => {
                    prop.name = "arg".to_string();
                    value_type = PropValueType::Var;
                    parse_state = PropParseState::Value;
                    iter.next();
                }
                _ => panic!("Unexpected `{{` "),
            },
            '}' => match parse_state {
                PropParseState::Value => match value_type {
                    PropValueType::Var => {
                        parse_state = PropParseState::Name;
                        iter.next();
                    }
                    _ => panic!("Unexpected `}}` "),
                },
                _ => panic!("Unexpected `}}` "),
            },
            '/' => match parse_state {
                PropParseState::Value => {
                    if let Some(next) = iter.next() {
                        buffer.push(next);
                    }
                }
                _ => break,
            },
            '>' => match parse_state {
                PropParseState::Name => break,
                _ => panic!("Unexpected `>`"),
            },
            char if !char.is_whitespace() => {
                if let Some(next) = iter.next() {
                    match parse_state {
                        PropParseState::Name => prop.name.push(next),
                        PropParseState::Value => buffer.push(next),
                        _ => panic!("Unexpected literal"),
                    }
                }
            }
            _ => match parse_state {
                PropParseState::Name => break,
                PropParseState::Value => {
                    if let Some(next) = iter.next() {
                        buffer.push(next)
                    }
                }
                _ => {
                    iter.next();
                }
            },
        }
    }
    prop.value = match value_type {
        PropValueType::None => None,
        PropValueType::Literal => Some(PropType::Literal(buffer)),
        PropValueType::Var => Some(PropType::Var(buffer)),
    };

    prop
}
