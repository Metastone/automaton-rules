mod compiler;

use compiler::lexer::Lexer;

fn main() {
    let mut lexer = Lexer::new("/home/metastone/Documents/projects/mutations/resources/game_of_life.txt").unwrap();
    loop {
        match lexer.get_next_token() {
            Ok(token) => {
                println!("{}", token);
            },
            Err(error) => {
                println!("{}", error);
                break;
            }
        }
    }
}
