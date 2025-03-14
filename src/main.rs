use std::fs;

use anyhow::Result;
use parser::start_parse;

mod asm;
mod ast;
mod parser;
mod iter;

fn main() -> Result<()> {
    let contents = fs::read_to_string("./file.html8")?;
    println!("{:?}", start_parse(contents));

    Ok(())
}
