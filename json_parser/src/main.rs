use crate::parser::Parse;
use crate::parser::Parser;
use crate::parser::ParserErr;
use crate::parser::ParserErrKind;
use crate::token::TokenKind;
use json_parser_macros::JsonDeserialise;

mod parser;
mod scanner;
mod token;

#[derive(JsonDeserialise, Debug)]
struct Test {
    name: Option<String>,
    age: i64,
}

fn main() {
    let result: Result<Test, _> = Parser::parse(include_str!("test_data/test_blob.json"));
    // let result = Test::parse_json(r#"{"name": "hi", "age": 3}"#);
    println!("{result:#?}");

    // let parsed = Parser::parse("").unwrap();
    // let result = match parsed {
    //     Any::Object(data) => data,
    //     _ => panic!("Not an object"),
    // };
    //
    // let result = result.props.get("").unwrap();
}
