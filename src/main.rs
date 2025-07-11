use scanner::Scanner;
use token::TokenKind;

mod scanner;
mod token;

fn main() {
    let mut scan = Scanner::init(include_str!("test.json"));

    let mut token = scan.next_token();
    while token.kind != TokenKind::EOF {
        println!("{token:?}");
        token = scan.next_token();
    }
}
