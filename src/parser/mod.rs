pub mod types;

use crate::{errors::parser::ParserError, iter::Iter};
use types::{ASTBody, ASTNode, ASTProp, PropType};

#[derive(Debug)]
enum ParseState {
    None,
    Tag,
    Props,
    Body,
    ClosingTag,
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

pub struct Parser {
    contents: String,
}

impl Parser {
    pub fn new(contents: String) -> Self {
        Self { contents }
    }

    pub fn parse(&self) -> ASTNode {
        let mut iter: Iter<char> = Iter::from(self.contents.chars());
        self._parse(&mut iter)
    }

    fn _parse(&self, iter: &mut Iter<char>) -> ASTNode {
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
                    Some('!') => Self::handle_comment(iter),
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
                                tag.children.push(ASTBody::Tag(Box::new(self._parse(iter))));
                            }
                        }
                        _ => ParserError::error("Unexpected `<` tag", iter),
                    },
                    None => ParserError::error("Unexpected EOF", iter),
                },
                '>' => match parse_state {
                    ParseState::Props | ParseState::Tag => parse_state = ParseState::Body,
                    ParseState::ClosingTag => {
                        (tag.name != closing_tag).then(|| {
                            ParserError::error(
                                &format!(
                                    "Unexpected closing tag: </{}>. Expected </{}>",
                                    closing_tag, tag.name
                                ),
                                iter,
                            );
                        });
                        return tag;
                    }
                    _ => ParserError::error("Unexpected `>` tag", iter),
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
                    ParseState::Props => tag.props.push(self.process_prop(iter)),
                    ParseState::ClosingTag => closing_tag.push(char),
                    ParseState::Body => buffer.push(char),
                    _ => ParserError::error("Unexpected literal", iter),
                },
                char if char.is_whitespace() => match parse_state {
                    ParseState::Tag => parse_state = ParseState::Props,
                    ParseState::Body => buffer.push(char),
                    _ => {}
                },
                _ => {}
            }
        }

        ParserError::error("Unexpected EOF", iter);
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

    fn process_prop(&self, iter: &mut Iter<char>) -> ASTProp {
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
                    _ => {
                        iter.next();
                        ParserError::error("Unexpected `=`", iter)
                    }
                },
                '"' => match parse_state {
                    PropParseState::Eq => {
                        value_type = PropValueType::Literal;
                        parse_state = PropParseState::Value;
                        iter.next();
                    }
                    PropParseState::Value => match value_type {
                        PropValueType::Literal => {
                            iter.next();
                            break;
                        }
                        _ => buffer.push(iter.next().unwrap()),
                    },
                    PropParseState::Name if prop.name.is_empty() => {
                        prop.name = "arg".to_string();
                        value_type = PropValueType::Literal;
                        parse_state = PropParseState::Value;
                        iter.next();
                    }
                    _ => {
                        iter.next();
                        ParserError::error("Unexpected `\"`", iter)
                    }
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
                    _ => {
                        iter.next();
                        ParserError::error("Unexpected `{`", iter)
                    }
                },
                '}' => match parse_state {
                    PropParseState::Value => match value_type {
                        PropValueType::Var => {
                            iter.next();
                            break;
                        }
                        _ => {
                            iter.next();
                            ParserError::error("Unexpected `}`", iter)
                        }
                    },
                    _ => {
                        iter.next();
                        ParserError::error("Unexpected `}`", iter)
                    }
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
                    _ => {
                        iter.next();
                        ParserError::error("Unexpected `>`", iter)
                    }
                },
                char if !char.is_whitespace() => {
                    if let Some(next) = iter.next() {
                        match parse_state {
                            PropParseState::Name => prop.name.push(next),
                            PropParseState::Value => buffer.push(next),
                            _ => ParserError::error("Unexpected literal", iter),
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
}
