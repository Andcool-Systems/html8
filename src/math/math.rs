use crate::iter::Iter;

#[derive(Debug, Clone)]
pub enum MathToken {
    Number(i64),
    Variable(String),
    Literal(String),
    Add(Box<MathToken>, Box<MathToken>),
    Sub(Box<MathToken>, Box<MathToken>),
    Mul(Box<MathToken>, Box<MathToken>),
    Div(Box<MathToken>, Box<MathToken>),
    Pow(Box<MathToken>, Box<MathToken>),
}

pub struct MathParser {
    iter: Iter<char>,
}

impl MathParser {
    pub fn new<I: IntoIterator<Item = char>>(iter: I) -> Self {
        Self {
            iter: Iter::from(iter),
        }
    }

    fn parse_primary(&mut self) -> MathToken {
        match self.iter.peek() {
            Some('.') => todo!("Float not yet implemented"),
            Some('(') | Some(')') => todo!("Parentheses not implemented yet"),
            Some('"') => self.process_literal(),
            Some(ch) if ch.is_whitespace() => {
                self.iter.next();
                self.parse_primary()
            }
            Some(ch) if ch.is_numeric() => self.process_number(),
            Some(ch) if ch.is_alphabetic() => self.process_var(),
            Some(ch) => panic!("Unexpected char: {}", ch),
            None => panic!("Unexpected EOI"),
        }
    }

    pub fn parse_expr(&mut self) -> MathToken {
        let mut node = self.parse_term();

        while let Some(char) = self.iter.peek() {
            match char {
                '+' | '-' => {
                    self.iter.next();
                    let r = self.parse_term();
                    node = match char {
                        '+' => MathToken::Add(Box::new(node), Box::new(r)),
                        '-' => MathToken::Sub(Box::new(node), Box::new(r)),
                        _ => unreachable!(),
                    };
                }
                ch if ch.is_whitespace() => {
                    self.iter.next();
                    continue;
                }
                _ => break,
            }
        }

        node
    }

    fn parse_term(&mut self) -> MathToken {
        let mut node = self.parse_exponent();

        while let Some(char) = self.iter.peek() {
            match char {
                '*' | '/' => {
                    self.iter.next();
                    let r = self.parse_exponent();
                    node = match char {
                        '/' => MathToken::Div(Box::new(node), Box::new(r)),
                        '*' => MathToken::Mul(Box::new(node), Box::new(r)),
                        _ => unreachable!(),
                    };
                }
                ch if ch.is_whitespace() => {
                    self.iter.next();
                    continue;
                }
                _ => break,
            }
        }

        node
    }

    fn parse_exponent(&mut self) -> MathToken {
        let mut node = self.parse_primary();

        while let Some(char) = self.iter.peek() {
            match char {
                '^' => {
                    self.iter.next();
                    let r = self.parse_primary();
                    node = MathToken::Pow(Box::new(node), Box::new(r));
                }
                ch if ch.is_whitespace() => {
                    self.iter.next();
                    continue;
                }
                _ => break,
            }
        }

        node
    }

    fn process_number(&mut self) -> MathToken {
        let mut buf = String::new();
        while matches!(self.iter.peek(), Some(ch) if ch.is_numeric()) {
            buf.push(self.iter.next().unwrap());
        }

        MathToken::Number(buf.parse().expect("Invalid number"))
    }

    fn process_var(&mut self) -> MathToken {
        let mut buf = String::new();
        while matches!(self.iter.peek(), Some(ch) if ch.is_alphanumeric()) {
            buf.push(self.iter.next().unwrap());
        }

        MathToken::Variable(buf)
    }

    fn process_literal(&mut self) -> MathToken {
        self.iter.next();
        let mut buf = String::new();
        while let Some(ch) = self.iter.peek() {
            match ch {
                '"' => {
                    return MathToken::Literal(buf);
                }
                ch => {
                    self.iter.next();
                    buf.push(ch)
                }
            }
        }

        panic!("Unclosed `\"` for literal `{}...`", buf);
    }
}
