/// This module provides lexical analysis functions

use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::fmt;

static DELIMITERS: [char; 5] = ['{', '}', '(', ')', ','];
static SINGLE_CHAR_OPERATORS: [char; 2] = ['<', '>'];
static TWO_CHAR_OPERATORS: [&str; 6] = ["&&", "||", "==", "!=", "<=", ">="];
static OPERATOR_FIRST_CHARS: [char; 6] = ['&', '|', '=', '!', '<', '>'];

pub struct Token {
    pub str: String,
    pub line: u32,
    pub column: u32
}

impl Token {
    fn new(str: String, lexer: &Lexer) -> Token {
        let column = if lexer.current_char_in_token { lexer.current_column } else { lexer.previous_column };
        let line = if lexer.current_char == '\n' { lexer.previous_line } else { lexer.current_line };
        Token {
            str,
            line,
            column
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\" - line {}, column {}", self.str, self.line, self.column)
    }
}

pub struct Lexer<'a> {
    reader: BufReader<File>,
    file_name: &'a str,
    previous_line: u32,
    previous_column: u32,
    current_line: u32,
    current_column: u32,
    current_char_in_token: bool,
    current_char: char
}

impl<'a> Lexer<'a> {
    pub fn new(file_name: &str) -> Result<Lexer, io::Error> {
        let file = File::open(file_name)?;
        let reader = BufReader::new(file);
        Ok(Lexer {
            reader,
            file_name,
            previous_line: 1,
            previous_column: 0,
            current_line: 1,
            current_column: 0,
            current_char_in_token: false,
            current_char: '\n'
        })
    }

    pub fn get_next_token(&mut self) -> Result<Token, String> {
        // Read until a not-whitespace parameter is found.
        let mut c = self.read_char()?;
        while c.is_ascii_whitespace() && c != '\u{0}' {
            c = self.read_char()?;
        }

        // The token is a single delimiter character.
        if DELIMITERS.contains(&c) {
            return Ok(Token::new(c.to_string(), &self));
        }

        // The token seems to be an operator.
        if OPERATOR_FIRST_CHARS.contains(&c) {
            return self.get_operator_token(c);
        }

        // The token should be a number or an alpha-numeric identifier (that doesn't start with a number).
        self.get_number_or_id_token(c)
    }

    fn get_operator_token(&mut self, c: char) -> Result<Token, String> {
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
            return Ok(Token::new(token, &self));
        }
        // The token is a single character operator ('<' or '>')
        else if (c == '<' || c == '>')
            && (c2.is_ascii_whitespace()
                || c2 == '\u{0}'
                || c2.is_ascii_alphanumeric()
                || DELIMITERS.contains(&c2)
                || OPERATOR_FIRST_CHARS.contains(&c2)) {
            token.pop();
            self.rewind_char()?;
            return Ok(Token::new(token, &self));
        }
        // The token starts as an operator but not one
        else {
            return Err(format!("Invalid token {}. Note : recognized operators are {:?} and {:?}.", Token::new(token, &self), SINGLE_CHAR_OPERATORS, TWO_CHAR_OPERATORS));
        }
    }

    fn get_number_or_id_token(&mut self, first_char: char) -> Result<Token, String> {
        let is_token_number = first_char.is_ascii_digit();
        let is_token_identifier = first_char.is_ascii_alphabetic();
        let mut rewind_one_char = false;
        let mut failure = false;

        let mut token = String::new();
        let mut c = first_char;
        let mut dot_encountered = false;

        while !c.is_ascii_whitespace() && c != '\u{0}' {
            token.push(c);

            if is_token_number && !c.is_ascii_digit() {
                if DELIMITERS.contains(&c) || OPERATOR_FIRST_CHARS.contains(&c) {
                    rewind_one_char = true;
                    break;
                } else if c == '.' && !dot_encountered {
                    dot_encountered = true;
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
                if is_token_number { Err(format!("Invalid token {}. It starts with a digit but is not a number.", Token::new(token, &self))) }
                else { Err(format!("Invalid token {}. It contains illegal characters.", Token::new(token, &self))) }
        }

        // No token found and we reached end-of-file
        if token.len() == 0 && c == '\u{0}' {
            return Ok(Token::new(String::new(), &self))
        }

        // The last character is nor part of the token, we just have to un-read it and we are good.
        if rewind_one_char {
            token.pop();
            self.rewind_char()?;
       }

        // Token is a valid number or identifier
        Ok(Token::new(token, &self))
    }

    fn read_char(&mut self) -> Result<char, String> {
        let mut buffer = [0; 1];
        match self.reader.read(&mut buffer) {
            Ok(_) => {
                self.current_char = buffer[0] as char;
                self.current_char_in_token = !(self.current_char.is_ascii_whitespace() || self.current_char == '\u{0}');
                if self.current_char == '\n' {
                    self.previous_line = self.current_line;
                    self.previous_column = self.current_column;
                    self.current_line += 1;
                    self.current_column = 0;
                } else {
                    self.previous_column = self.current_column;
                    self.current_column += 1;
                }
            }
            Err(e) => {
                return Err(format!("Cannot read character from file {} (line {}, column {}). Cause : {:?}",
                                   self.file_name, self.current_line, self.current_column, e))
            }
        }
        Ok(buffer[0] as char)
    }

    fn rewind_char(&mut self) -> Result<(), String> {
        if let Err(error) = self.reader.seek(SeekFrom::Current(-1)) {
            return Err(format!("Could not get token (line {}, column {}). Cause : {:?}",
                        self.previous_line, self.previous_column, error));
        }
        if self.current_char == '\n' {
            self.current_line = self.previous_line;
        }
        self.current_column = self.previous_column;
        self.current_char_in_token = true;
        Ok(())
    }
}

// TODO Add tests for line and columns feature, current tests are not enough
#[cfg(test)]
mod tests {
    use crate::compiler::lexer::{Lexer, SINGLE_CHAR_OPERATORS, TWO_CHAR_OPERATORS};

    static BENCH_NICE_FILE: &str = "resources/tests/lexer_benchmark_nice.txt";
    static BENCH_UGLY_FILE: &str = "resources/tests/lexer_benchmark_ugly.txt";
    static NON_EXISTING_FILE: &str = "resources/tests/does_not_exist.txt";
    static OPERATOR_TYPO_FILE: &str = "resources/tests/lexer_operator_typo.txt";
    static NB_WITH_TWO_DOTS_FILE: &str = "resources/tests/lexer_number_with_two_dots.txt";
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
        assert_eq!(lexer.get_next_token().unwrap().str, "th15I5AnAlphanum3r1cId3nt1f1er");
        assert_eq!(lexer.get_next_token().unwrap().str, "thisTooAndNextUpIsANumber");
        assert_eq!(lexer.get_next_token().unwrap().str, "123456");
        assert_eq!(lexer.get_next_token().unwrap().str, "<");
        assert_eq!(lexer.get_next_token().unwrap().str, ">");
        assert_eq!(lexer.get_next_token().unwrap().str, "test");
        assert_eq!(lexer.get_next_token().unwrap().str, "<=");
        assert_eq!(lexer.get_next_token().unwrap().str, ">=");
        assert_eq!(lexer.get_next_token().unwrap().str, "&&");
        assert_eq!(lexer.get_next_token().unwrap().str, "||");
        assert_eq!(lexer.get_next_token().unwrap().str, "==");
        assert_eq!(lexer.get_next_token().unwrap().str, "!=");
        assert_eq!(lexer.get_next_token().unwrap().str, "test");
        assert_eq!(lexer.get_next_token().unwrap().str, ",");
        assert_eq!(lexer.get_next_token().unwrap().str, "test");
        assert_eq!(lexer.get_next_token().unwrap().str, "(");
        assert_eq!(lexer.get_next_token().unwrap().str, ")");
        assert_eq!(lexer.get_next_token().unwrap().str, "{");
        assert_eq!(lexer.get_next_token().unwrap().str, "}");
        assert_eq!(lexer.get_next_token().unwrap().str, "3.14");
        assert_eq!(lexer.get_next_token().unwrap().str, "test");
        assert!(lexer.get_next_token().unwrap().str.is_empty());
        assert!(lexer.get_next_token().unwrap().str.is_empty());
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
                "Invalid token \"|-\" - line 1, column 2. Note : recognized operators are {:?} and {:?}.", SINGLE_CHAR_OPERATORS, TWO_CHAR_OPERATORS)),
            _ => assert!(false),
        }
        assert_eq!(lexer.get_next_token().unwrap().str, "thisTokenShouldBeReadWithoutIssues");
    }

    #[test]
    fn tokenize_number_with_two_dots_fails() {
        let mut lexer = Lexer::new(NB_WITH_TWO_DOTS_FILE).unwrap();
        match lexer.get_next_token() {
            Err(error) => assert_eq!(error, "Invalid token \"1.000.000\" - line 1, column 9. It starts with a digit but is not a number."),
            _ => assert!(false),
        }
        assert_eq!(lexer.get_next_token().unwrap().str, "thisTokenShouldBeReadWithoutIssues");
    }

    #[test]
    fn tokenize_number_with_alphabetic_fails() {
        let mut lexer = Lexer::new(NB_WITH_ALPHABETIC_FILE).unwrap();
        match lexer.get_next_token() {
            Err(error) => assert_eq!(error, "Invalid token \"10O0\" - line 1, column 4. It starts with a digit but is not a number."),
            _ => assert!(false),
        }
        assert_eq!(lexer.get_next_token().unwrap().str, "thisTokenShouldBeReadWithoutIssues");
    }

    #[test]
    fn tokenize_id_with_illegal_char_fails() {
        let mut lexer = Lexer::new(ID_WITH_ILLEGAL_CHAR_FILE).unwrap();
        match lexer.get_next_token() {
            Err(error) => assert_eq!(error, "Invalid token \"hello_world\" - line 1, column 11. It contains illegal characters."),
            _ => assert!(false),
        }
        assert_eq!(lexer.get_next_token().unwrap().str, "thisTokenShouldBeReadWithoutIssues");
    }
}
