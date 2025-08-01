use json_parser::*;
use json_parser_macros::JsonDeserialise;

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
    let result = Parser::parse::<Root>(include_str!("test_data/test_blob.json"));
    println!("{result:#?}");
}
