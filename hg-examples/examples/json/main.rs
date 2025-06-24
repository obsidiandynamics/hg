use hg::lexer;
use hg::symbols::SymbolTable;

pub fn parse(str: &str) {
    let _tokens = lexer::Tokeniser::new(str, SymbolTable::default());
}

fn main() {
    println!("hello json");
    
    
}