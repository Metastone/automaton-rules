mod compiler;

use compiler::parser;

fn main() {
    match parser::parse("/home/metastone/Documents/projects/mutations/resources/game_of_life.txt") {
        Ok(_) => { println!("Success"); },
        Err(error) => { println!("{}", error); }
    }
}
