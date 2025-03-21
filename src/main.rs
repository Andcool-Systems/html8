use std::io::Write;
use std::process::exit;
use std::{fs, path::Path, process::Command};

use anyhow::Result;
use code_tree::start_generating_code_tree;
use compiler::compiler::CLang;
use parser::start_parse;

mod code_tree;
mod compiler;
mod definitions;
mod iter;
mod libs;
mod math;
mod parser;
mod types;

fn main() -> Result<()> {
    let contents = fs::read_to_string("./example.html8")?;
    let tree = start_parse(contents);
    let code_tree = start_generating_code_tree(tree);
    let mut comp = CLang::new(code_tree);
    let code = comp.compile();

    let dir_path = "output";
    let file_path = format!("{}/code.c", dir_path);
    let out_path = format!("{}/code", dir_path);

    if !Path::new(dir_path).exists() {
        fs::create_dir(dir_path)?;
    }

    let mut file = fs::File::create(&file_path)?;
    writeln!(file, "{}", code)?;

    let compile_out = Command::new("gcc")
        .args(vec![&file_path, "-o", &out_path, "-lm"])
        .output()?;

    if !compile_out.status.success() {
        println!(
            "Compilation failed: {}",
            String::from_utf8_lossy(&compile_out.stderr)
        );
        exit(-1)
    }

    println!(
        "{}",
        String::from_utf8_lossy(&Command::new("./output/code").output()?.stdout)
    );

    Ok(())
}
