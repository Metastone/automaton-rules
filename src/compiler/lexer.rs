use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Seek, SeekFrom};

static DELIMITERS: [char; 5] = ['{', '}', '(', ')', ','];
static SINGLE_CHAR_OPERATORS: [char; 2] = ['<', '>'];
static TWO_CHAR_OPERATORS: [&str; 6] = ["&&", "||", "==", "!=", "<=", ">="];
static OPERATOR_FIRST_CHARS: [char; 6] = ['&', '|', '=', '!', '<', '>'];

pub struct Lexer<'a> {
    reader: BufReader<File>,
    file_name: &'a str
}

impl<'a> Lexer<'a> {
    pub fn new(file_name: &str) -> Result<Lexer, io::Error> {
        let file = File::open(file_name)?;
        let reader = BufReader::new(file);
        Ok(Lexer {
            reader,
            file_name
        })
    }

    pub fn get_next_token(&mut self) -> Result<String, String> {
        // Read until a not-whitespace parameter is found.
        let mut c = self.read_char()?;
        while c.is_ascii_whitespace() && c != '\u{0}' {
            c = self.read_char()?;
        }

        // The token is a single delimiter character.
        if DELIMITERS.contains(&c) {
            return Ok(c.to_string());
        }

        // The token seems to be an operator.
        if OPERATOR_FIRST_CHARS.contains(&c) {
            return self.get_operator_token(c);
        }

        // The token should be a number or an alpha-numeric identifier (that doesn't start with a number).
        self.get_number_or_id_token(c)
    }

    fn get_operator_token(&mut self, c: char) -> Result<String, String> {
        let mut token = String::new();
        token.push(c);
        let expected_char = match c {
            '!' => '=',
            '<' => '=',
            '>' => '=',
            _ => c
        };
        let c2 = self.read_char()?;
        token.push(c2);

        // The token is a two-characters operator
        if c2 == expected_char {
            return Ok(token);
        }
        // The token is a single character operator ('<' or '>')
        else if (c == '<' || c == '>')
            && (c2.is_ascii_whitespace()
                || c2 == '\u{0}'
                || c2.is_ascii_alphanumeric()
                || DELIMITERS.contains(&c2)
                || OPERATOR_FIRST_CHARS.contains(&c2)) {
            token.pop();
            if let Err(error) = self.reader.seek(SeekFrom::Current(-1)) {
                return Err(format!("Could not get token. Cause : {:?}", error));
            }
            return Ok(token);
        }
        // The token starts as an operator but not one
        else {
            return Err(format!("Invalid token {}. Note : recognized operators are {:?} and {:?}.", token, SINGLE_CHAR_OPERATORS, TWO_CHAR_OPERATORS));
        }
    }

    fn get_number_or_id_token(&mut self, first_char: char) -> Result<String, String> {
        let is_token_number = first_char.is_ascii_digit();
        let is_token_identifier = first_char.is_ascii_alphabetic();
        let mut rewind_one_char = false;
        let mut failure = false;

        let mut token = String::new();
        let mut c = first_char;

        while !c.is_ascii_whitespace() && c != '\u{0}' {
            token.push(c);

            if is_token_number && !c.is_ascii_digit() {
                if DELIMITERS.contains(&c) || OPERATOR_FIRST_CHARS.contains(&c) {
                    rewind_one_char = true;
                    break;
                } else {
                    failure = true;
                }
            }

            if is_token_identifier && !c.is_ascii_alphanumeric() {
                if DELIMITERS.contains(&c) || OPERATOR_FIRST_CHARS.contains(&c) {
                    rewind_one_char = true;
                    break;
                } else {
                    failure = true;
                }
            }

            c = self.read_char()?;
        }

        // The token is not a valid number or identifier
        if failure {
            return
                if is_token_number { Err(format!("Invalid token {}. It starts with a digit but is not a number.", token)) }
                else { Err(format!("Invalid token {}. It contains illegal characters.", token)) }
        }

        // No token found and we reached end-of-file
        if token.len() == 0 && c == '\u{0}' {
            return Ok(String::new())
        }

        // The last character is nor part of the token, we just have to remove it and we are good.
        if rewind_one_char {
            token.pop();
            if let Err(error) = self.reader.seek(SeekFrom::Current(-1)) {
                return Err(format!("Could not get token. Cause : {:?}", error));
            }
        }

        // Token is a valid number or identifier
        Ok(token)
    }

    fn read_char(&mut self) -> Result<char, String> {
        let mut buffer = [0; 1];
        match self.reader.read(&mut buffer) {
            Ok(_) => return Ok(buffer[0] as char),
            Err(e) => return Err(format!("Cannot read character from file {}. Cause : {:?}", self.file_name, e))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::compiler::lexer::{Lexer, SINGLE_CHAR_OPERATORS, TWO_CHAR_OPERATORS};

    static BENCH_NICE_FILE: &str = "resources/tests/lexer_benchmark_nice.txt";
    static BENCH_UGLY_FILE: &str = "resources/tests/lexer_benchmark_ugly.txt";
    static NON_EXISTING_FILE: &str = "resources/tests/does_not_exist.txt";
    static OPERATOR_TYPO_FILE: &str = "resources/tests/lexer_operator_typo.txt";
    static NB_WITH_ILLEGAL_CHAR_FILE: &str = "resources/tests/lexer_number_with_illegal_char.txt";
    static NB_WITH_ALPHABETIC_FILE: &str = "resources/tests/lexer_number_with_alphabetic.txt";
    static ID_WITH_ILLEGAL_CHAR_FILE: &str = "resources/tests/lexer_id_with_illegal_char.txt";

    #[test]
    fn tokenize_benchmark_nice_succeeds() {
        let mut lexer = Lexer::new(BENCH_NICE_FILE).unwrap();
        check_benchmark_output(&mut lexer);
    }

    #[test]
    fn tokenize_benchmark_ugly_succeeds() {
        let mut lexer = Lexer::new(BENCH_UGLY_FILE).unwrap();
        check_benchmark_output(&mut lexer);
    }

    fn check_benchmark_output(lexer: &mut Lexer) {
        assert_eq!(lexer.get_next_token().unwrap(), "th15I5AnAlphanum3r1cId3nt1f1er");
        assert_eq!(lexer.get_next_token().unwrap(), "thisTooAndNextUpIsANumber");
        assert_eq!(lexer.get_next_token().unwrap(), "123456");
        assert_eq!(lexer.get_next_token().unwrap(), "<");
        assert_eq!(lexer.get_next_token().unwrap(), ">");
        assert_eq!(lexer.get_next_token().unwrap(), "test");
        assert_eq!(lexer.get_next_token().unwrap(), "<=");
        assert_eq!(lexer.get_next_token().unwrap(), ">=");
        assert_eq!(lexer.get_next_token().unwrap(), "&&");
        assert_eq!(lexer.get_next_token().unwrap(), "||");
        assert_eq!(lexer.get_next_token().unwrap(), "==");
        assert_eq!(lexer.get_next_token().unwrap(), "!=");
        assert_eq!(lexer.get_next_token().unwrap(), "test");
        assert_eq!(lexer.get_next_token().unwrap(), ",");
        assert_eq!(lexer.get_next_token().unwrap(), "test");
        assert_eq!(lexer.get_next_token().unwrap(), "(");
        assert_eq!(lexer.get_next_token().unwrap(), ")");
        assert_eq!(lexer.get_next_token().unwrap(), "{");
        assert_eq!(lexer.get_next_token().unwrap(), "}");
        assert_eq!(lexer.get_next_token().unwrap(), "test");
        assert!(lexer.get_next_token().unwrap().is_empty());
        assert!(lexer.get_next_token().unwrap().is_empty());
   }

    #[test]
    fn tokenize_no_file_fails() {
        match Lexer::new(NON_EXISTING_FILE) {
            Err(io_error) => assert!(io_error.to_string().contains("No such file or directory")),
            _ => assert!(false),
        }
    }

    #[test]
    fn tokenize_operator_typo_fails() {
        let mut lexer = Lexer::new(OPERATOR_TYPO_FILE).unwrap();
        match lexer.get_next_token() {
            Err(error) => assert_eq!(error, format!(
                "Invalid token |-. Note : recognized operators are {:?} and {:?}.", SINGLE_CHAR_OPERATORS, TWO_CHAR_OPERATORS)),
            _ => assert!(false),
        }
        assert_eq!(lexer.get_next_token().unwrap(), "thisTokenShouldBeReadWithoutIssues");
    }

    #[test]
    fn tokenize_number_with_illegal_char_fails() {
        let mut lexer = Lexer::new(NB_WITH_ILLEGAL_CHAR_FILE).unwrap();
        match lexer.get_next_token() {
            Err(error) => assert_eq!(error, "Invalid token 3.14. It starts with a digit but is not a number."),
            _ => assert!(false),
        }
        assert_eq!(lexer.get_next_token().unwrap(), "thisTokenShouldBeReadWithoutIssues");
    }

    #[test]
    fn tokenize_number_with_alphabetic_fails() {
        let mut lexer = Lexer::new(NB_WITH_ALPHABETIC_FILE).unwrap();
        match lexer.get_next_token() {
            Err(error) => assert_eq!(error, "Invalid token 10O0. It starts with a digit but is not a number."),
            _ => assert!(false),
        }
        assert_eq!(lexer.get_next_token().unwrap(), "thisTokenShouldBeReadWithoutIssues");
    }

    #[test]
    fn tokenize_id_with_illegal_char_fails() {
        let mut lexer = Lexer::new(ID_WITH_ILLEGAL_CHAR_FILE).unwrap();
        match lexer.get_next_token() {
            Err(error) => assert_eq!(error, "Invalid token hello_world. It contains illegal characters."),
            _ => assert!(false),
        }
        assert_eq!(lexer.get_next_token().unwrap(), "thisTokenShouldBeReadWithoutIssues");
    }
}
