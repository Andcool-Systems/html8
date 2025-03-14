use std::fs;

use anyhow::Result;
use code_tree::process_code_tree;
use parser::start_parse;

mod code_tree;
mod iter;
mod parser;

fn main() -> Result<()> {
    let contents = fs::read_to_string("./file.html8")?;
    let tree = start_parse(contents);
    println!("{:?}", process_code_tree(tree));

    Ok(())
}
