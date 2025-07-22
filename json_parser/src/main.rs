use crate::parser::Parse;
use crate::parser::Parser;
use crate::parser::ParserErr;
use crate::parser::ParserErrKind;
use crate::token::TokenKind;
use json_parser_macros::JsonDeserialise;

mod parser;
mod scanner;
mod token;

#[derive(Debug, JsonDeserialise)]
pub struct Root {
    pub name: String,
    pub age: u32,
    pub is_verified: bool,
    pub balance: f64,
    pub nickname: Option<String>,
    pub contact: Contact,
    pub preferences: Preferences,
    pub tags: Vec<String>,
    pub history: Vec<History>,
    pub unicode_example: String,
    pub numbers: Numbers,
}

#[derive(Debug, JsonDeserialise)]
pub struct Contact {
    pub email: String,
    pub phone: String,
    pub address: Address,
}

#[derive(Debug, JsonDeserialise)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub zipcode: String,
    pub country: String,
}

#[derive(Debug, JsonDeserialise)]
pub struct Preferences {
    pub notifications: Notifications,
    pub theme: String,
    pub language: String,
}

#[derive(Debug, JsonDeserialise)]
pub struct Notifications {
    pub email: bool,
    pub sms: bool,
}

#[derive(Debug, JsonDeserialise)]
pub struct History {
    pub login: String,
    pub ip: String,
    pub success: bool,
}

#[derive(Debug, JsonDeserialise)]
pub struct Numbers {
    pub int: i64,
    pub float: f64,
    pub scientific: f64,
    pub scientific_no_decimal: f64,
    pub negative: i64,
    pub negative_scientific: f64,
}

fn main() {
    let result: Result<Root, _> = Parser::parse(include_str!("test_data/test_blob.json"));
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
