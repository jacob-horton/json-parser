use parser::Parser;

mod parser;
mod scanner;
mod token;

fn main() {
    let result = Parser::parse(include_str!("test.json"));
    println!("{result:#?}");
}
