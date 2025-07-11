use scanner::{Scanner, ScannerErr, ScannerErrKind};

mod scanner;
mod token;

fn main() {
    let mut scan = Scanner::init(include_str!("test.json"));
    // let mut scan = Scanner::init("123a");

    let mut token = scan.next_token();
    while let Ok(t) = token {
        println!("{t:?}");
        token = scan.next_token();
    }

    match token {
        Err(ScannerErr {
            kind: ScannerErrKind::EndOfSource,
            line: _,
            lexeme: _,
        }) => {
            println!("Finished scanning");
        }
        Err(e) => {
            println!("{e:#?}");
        }
        Ok(_) => panic!("[BUG] Should not get here without error"),
    };
}
