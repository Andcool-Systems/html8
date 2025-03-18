use std::fs;

use anyhow::Result;
use code_tree::start_generating_code_tree;
use parser::start_parse;

mod code_tree;
mod definitions;
mod iter;
mod math;
mod parser;
mod types;

fn main() -> Result<()> {
    let contents = fs::read_to_string("./file.html8")?;
    let tree = start_parse(contents);
    println!("{:?}", start_generating_code_tree(tree));

    Ok(())
}
