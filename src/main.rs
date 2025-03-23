use crate::backends_loader::load_backends;
use crate::parser::types::ASTNode;
use anyhow::Result;
use code_tree::start_generating_code_tree;
use compiler_core::backend::Backend;
use compiler_core::types::NodeType;
use parser::start_parse;
use std::fs;
use std::process::exit;

mod backends_loader;
mod code_tree;
mod parser;
mod types;

fn main() -> Result<()> {
    let contents: String = fs::read_to_string("./example.html8")?;
    let tree: ASTNode = start_parse(contents);
    let code_tree: NodeType = start_generating_code_tree(tree);

    let mut backends = load_backends("./backends");

    if backends.is_empty() {
        println!("Не найдено бэкендов в './backends'");
        exit(1);
    }

    if !fs::exists("./output").expect("TODO: panic message") {
        fs::create_dir("./output").expect("TODO: panic message");
    }

    if let Some(backend) = backends.get_mut("C-lang") {
        backend.generate_code(&code_tree);
        backend.save_code(None).expect("TODO: panic message");
        backend.compile().expect("TODO: panic message");
        println!("{}", backend.run().expect("TODO: panic message"));
    }

    Ok(())
}
