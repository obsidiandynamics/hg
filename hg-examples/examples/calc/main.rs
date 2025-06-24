use crate::analyser::analyse;
use crate::ast::Eval;
use hg::lexer::Tokeniser;
use hg::parser::parse;
use hg::symbols::SymbolTable;
use std::error::Error;

mod analyser;
mod ast;

fn main() -> Result<(), Box<dyn Error>>{
    let str = r#"1 * 2 + 4 + 3 * 2 + 5"#;
    let tok = Tokeniser::new(str, SymbolTable::default());
    let root = parse(tok)?;
    let expr = analyse(root)?;
    let eval = expr.eval();
    println!(">> {str}");
    println!("<< {eval}");
    Ok(())
}