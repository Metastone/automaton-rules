mod compiler;

use compiler::semantic;

fn main() {
    match semantic::parse("/home/metastone/Documents/projects/mutations/resources/game_of_life.txt") {
        Ok(_) => { println!("Success"); },
        Err(errors) => {
            for i in 0..errors.len() {
                println!("ERROR : {}", errors[i]);
            }
        }
    }
}
