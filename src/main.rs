use std::any::Any;

use parser::Parser;
use scanner::{Scanner, ScannerErr, ScannerErrKind};

mod parser;
mod scanner;
mod token;

fn main() {
    let result = Parser::parse(include_str!("test.json"));
    println!("{result:#?}");

    // let mut token = scan.next_token();
    // while let Ok(t) = token {
    //     println!("{t:?}");
    //     token = scan.next_token();
    // }
    //
    // match token {
    //     Err(ScannerErr {
    //         kind: ScannerErrKind::EndOfSource,
    //         line: _,
    //         lexeme: _,
    //     }) => {
    //         println!("Finished scanning");
    //     }
    //     Err(e) => {
    //         println!("{e:#?}");
    //     }
    //     Ok(_) => panic!("[BUG] Should not get here without error"),
    // };
}
