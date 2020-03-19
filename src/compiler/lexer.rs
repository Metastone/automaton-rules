use std::fs::File;
use std::io;
use std::io::{BufReader, Read};

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
        let mut token = String::new();
        let mut c = self.read_char()?;

        // Read until a not-whitespace parameter is found.
        while c.is_ascii_whitespace() && c != '\u{0}' {
            c = self.read_char()?;
        }

        let is_token_integer = c.is_ascii_digit();

        // Read token. Stop at whitespace parameter.
        while !c.is_ascii_whitespace() && c != '\u{0}' {
            token.push(c);

            if token.len() == 1 && c == '{' || c == '}' || c == '(' || c == ')' || c == ',' {
                return Ok(token);
            }

            if token.len() == 1 && c == '<' || c == '>' {
                c = self.read_char()?;
                if c == '=' {
                    token.push(c);
                    return Ok(token);
                } else if c.is_ascii_whitespace() {
                    return Ok(token);
                } else {
                    token.push(c);
                    return Err(format!("Invalid token {}. Close valid tokens are '<', '>', '<=', and '>='.", token))
                }
            }

            if token.len() == 1 && c == '=' || c == '!' {
                c = self.read_char()?;
                token.push(c);
                if c == '=' {
                    return Ok(token);
                } else {
                    return Err(format!("Invalid token {}. Close valid tokens are '!=', '=='", token));
                }
            }

            if token.len() == 1 && c == '&' {
                c = self.read_char()?;
                token.push(c);
                if c == '&' {
                    return Ok(token);
                } else {
                    return Err(format!("Invalid token {}. Closest valid token is '&&'.", token));
                }
            }

            if token.len() == 1 && c == '|' {
                c = self.read_char()?;
                token.push(c);
                if c == '|' {
                    return Ok(token);
                } else {
                    return Err(format!("Invalid token {}. Closest valid token is '||'.", token));
                }
            }

            if !c.is_ascii_alphanumeric() {
                return Err(format!("Invalid token {}. Only unary operators, boolean operators and alphanumeric characters are allowed.", token.as_bytes()[0] as u8));
            }

            if is_token_integer && !c.is_ascii_digit() {
                return Err(format!("Invalid token {}. It starts with a digit but contains letters.", token));
            }

            c = self.read_char()?;
        }

        if token.len() == 0 && c == '\u{0}' {
            return Err("No token available, end of file reached.".to_string());
        }

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
