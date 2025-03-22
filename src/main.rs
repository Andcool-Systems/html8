use std::io::Write;
use std::process::{exit, Output};
use std::{fs, path::Path, process::Command};

use crate::{
    code_tree::types::NodeType, compiler::CLang, compiler::CompilerCodegen, parser::types::ASTNode,
};
use anyhow::Result;
use code_tree::start_generating_code_tree;
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
    let contents: String = fs::read_to_string("./example.html8")?;
    let tree: ASTNode = start_parse(contents);
    let code_tree: NodeType = start_generating_code_tree(tree);
    let mut comp = CLang::new(code_tree);
    let code: String = comp.compile();

    let dir_path: &str = "output";
    let file_path: String = format!("{}/code.c", dir_path);
    let out_path: String = format!("{}/code", dir_path);

    (!Path::new(dir_path).exists()).then(|| fs::create_dir(dir_path).map_err(|_| ()));

    // println!("{}", code);

    let mut file: fs::File = fs::File::create(&file_path)?;
    writeln!(file, "{}", code)?;

    let args: Vec<&str> = vec![
        &file_path,
        "-o",
        &out_path,
        "-w",
        "-std=gnu99",
        "-Wimplicit-int",
    ];
    let compiler: &str = "clang"; // or gcc

    /* println!(
        "{} {}",
        compiler,
        args.iter()
            .map(|arg: &&str| arg.to_string())
            .collect::<String>()
    ); */

    let compile_out: Output = Command::new(compiler).args(&args).output()?;

    (!compile_out.status.success()).then(|| {
        println!(
            "Compilation failed.\n\
            Command: {} {}\n\
            Output:\n{}",
            compiler,
            args.join(" "),
            String::from_utf8_lossy(&compile_out.stderr)
        );
        exit(-1)
    });

    println!(
        "{}",
        String::from_utf8_lossy(&Command::new("./output/code").output()?.stdout)
    );

    Ok(())
}
