use crate::iter::Iter;
use colored::Colorize;
use std::process;

pub struct ParserError {}
const T_SYMBOL: char = '┬';
const L_SYMBOL: char = '└';
const J_SYMBOL: char = '┘';
const D_SYMBOL: char = '─';

impl ParserError {
    fn trimmed_chars(s: &str) -> usize {
        s.len() - s.trim_start().len()
    }

    pub fn error(mess: &str, iter: &mut Iter<char>) -> ! {
        let mut error_char = iter.pos;
        let mut buf = String::new();
        while let Some(c) = iter.step_back() {
            if c == '\n' {
                break;
            }
        }
        let iter_pos = iter.pos;
        error_char -= iter.pos + 2;
        iter.next();

        while let Some(c) = iter.next() {
            if c == '\n' {
                break;
            }
            buf.push(c);
        }

        let _error_char = error_char - Self::trimmed_chars(&buf);
        buf = buf.trim().to_owned();
        let (left, char, right) = (
            &buf[.._error_char],
            &buf[_error_char.._error_char + 1],
            &buf[_error_char + 1..],
        );

        let line_no = iter.vec.iter().collect::<String>()[0..iter_pos]
            .chars()
            .filter(|c| c.eq(&'\n'))
            .count();

        let error_str = if _error_char >= mess.len() + 2 {
            Self::compile_left(mess, _error_char)
        } else {
            Self::compile_right(mess, _error_char)
        };

        eprintln!(
            "{}{}{}\n{}\n\n{} {}",
            left,
            char.red(),
            right,
            error_str,
            "Error at line".red().bold(),
            format!("{}:{}", line_no + 2, error_char + 1).yellow()
        );
        process::exit(-1);
    }

    fn compile_left(msg: &str, char_pos: usize) -> String {
        let offset = char_pos - (msg.len() + 2);
        format!(
            "{}{}\n{}{} {}{}",
            " ".repeat(char_pos),
            T_SYMBOL,
            " ".repeat(offset),
            msg.red().bold(),
            D_SYMBOL,
            J_SYMBOL
        )
    }

    fn compile_right(msg: &str, char_pos: usize) -> String {
        let offset = char_pos;
        format!(
            "{}{}\n{}{}{} {}",
            " ".repeat(char_pos),
            T_SYMBOL,
            " ".repeat(offset),
            L_SYMBOL,
            D_SYMBOL,
            msg.red().bold(),
        )
    }
}
