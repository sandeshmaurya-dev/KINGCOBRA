use crate::token::Token;

/// The Lexer is responsible for reading the source code character by character
/// and grouping them into meaningful 'Tokens'.
pub struct Lexer {
    input: Vec<char>,
    position: usize,      // Current position in input (points to current char)
    read_position: usize, // Current reading position (after current char)
    ch: char,             // Current character being examined
    pub line: usize,
    pub column: usize,
}

impl Lexer {
    /// Creates a new Lexer instance from a string of Kingcobra source code
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer {
            input: input.chars().collect(),
            position: 0,
            read_position: 0,
            ch: '\0',
            line: 1,
            column: 0,
        };
        lexer.read_char();
        lexer
    }

    /// Advances the lexer to the next character in the input
    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0'; // NUL character signifies End of File
        } else {
            self.ch = self.input[self.read_position];
            
            if self.ch == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    /// Peeks at the next character without advancing the position
    fn peek_char(&self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input[self.read_position]
        }
    }

    /// Gets the next token from the input stream
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let tok = match self.ch {
            '=' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::Eq
                } else {
                    Token::Assign
                }
            }
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Asterisk,
            '/' => Token::Slash,
            '<' => Token::Lt,
            '>' => Token::Gt,
            ':' => Token::Colon,
            ',' => Token::Comma,
            '.' => Token::Dot,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '!' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::NotEq
                } else {
                    Token::Illegal
                }
            }
            '"' => Token::String(self.read_string()),
            '\0' => Token::Eof,
            _ => {
                if is_letter(self.ch) {
                    let ident = self.read_identifier();
                    // We must return early here because read_identifier advances the character
                    return lookup_ident(&ident);
                } else if is_digit(self.ch) {
                    return Token::Int(self.read_number());
                } else {
                    Token::Illegal
                }
            }
        };

        self.read_char();
        tok
    }

    /// Skips over any spaces, tabs, or newlines
    fn skip_whitespace(&mut self) {
        while self.ch.is_ascii_whitespace() {
            self.read_char();
        }
    }

    /// Reads an entire identifier (like a variable name)
    fn read_identifier(&mut self) -> String {
        let position = self.position;
        while is_letter(self.ch) {
            self.read_char();
        }
        self.input[position..self.position].iter().collect()
    }

    /// Reads an entire number
    fn read_number(&mut self) -> i64 {
        let position = self.position;
        while is_digit(self.ch) {
            self.read_char();
        }
        let num_str: String = self.input[position..self.position].iter().collect();
        num_str.parse::<i64>().unwrap_or(0)
    }

    /// Reads a string literal
    fn read_string(&mut self) -> String {
        let position = self.position + 1; // skip the opening quote
        loop {
            self.read_char();
            if self.ch == '"' || self.ch == '\0' {
                break;
            }
        }
        self.input[position..self.position].iter().collect()
    }
}

/// Helper function to check if a character is valid in a variable name
fn is_letter(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_'
}

/// Helper function to check if a character is a number
fn is_digit(ch: char) -> bool {
    ch.is_ascii_digit()
}

/// Maps a string to either a Keyword Token or an Identifier Token
fn lookup_ident(ident: &str) -> Token {
    match ident {
        "let" => Token::Let,
        "fn" => Token::Fn,
        "when" => Token::When,
        "otherwise" => Token::Otherwise,
        "loop" => Token::Loop,
        "while" => Token::While,
        "import" => Token::Import,
        "class" => Token::Class,
        "new" => Token::New,
        "return" => Token::Return,
        "done" => Token::Done,
        "true" => Token::True,
        "false" => Token::False,
        _ => Token::Ident(ident.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_token() {
        let input = r#"let five = 5
let ten = 10

let add = fn(x, y):
    return x + y
done

let result = add(five, ten)

when 5 < 10:
    return true
otherwise:
    return false
done

10 == 10
10 != 9
"test string"
"#;

        let tests = vec![
            Token::Let,
            Token::Ident("five".to_string()),
            Token::Assign,
            Token::Int(5),
            Token::Let,
            Token::Ident("ten".to_string()),
            Token::Assign,
            Token::Int(10),
            Token::Let,
            Token::Ident("add".to_string()),
            Token::Assign,
            Token::Fn,
            Token::LParen,
            Token::Ident("x".to_string()),
            Token::Comma,
            Token::Ident("y".to_string()),
            Token::RParen,
            Token::Colon,
            Token::Return,
            Token::Ident("x".to_string()),
            Token::Plus,
            Token::Ident("y".to_string()),
            Token::Done,
            Token::Let,
            Token::Ident("result".to_string()),
            Token::Assign,
            Token::Ident("add".to_string()),
            Token::LParen,
            Token::Ident("five".to_string()),
            Token::Comma,
            Token::Ident("ten".to_string()),
            Token::RParen,
            Token::When,
            Token::Int(5),
            Token::Lt,
            Token::Int(10),
            Token::Colon,
            Token::Return,
            Token::True,
            Token::Otherwise,
            Token::Colon,
            Token::Return,
            Token::False,
            Token::Done,
            Token::Int(10),
            Token::Eq,
            Token::Int(10),
            Token::Int(10),
            Token::NotEq,
            Token::Int(9),
            Token::String("test string".to_string()),
            Token::Eof,
        ];

        let mut lexer = Lexer::new(input);

        for (i, expected_token) in tests.iter().enumerate() {
            let tok = lexer.next_token();
            assert_eq!(&tok, expected_token, "tests[{}] - token type wrong. expected={:?}, got={:?}", i, expected_token, tok);
        }
    }

    #[test]
    fn test_illegal_characters() {
        // Testing that the lexer correctly identifies completely unknown characters
        let input = "@#$";
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next_token(), Token::Illegal);
        assert_eq!(lexer.next_token(), Token::Illegal);
        assert_eq!(lexer.next_token(), Token::Illegal);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_empty_input() {
        // Testing that the lexer correctly ignores pure whitespace and returns EOF
        let input = "    \n\t  ";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Eof);
    }
}
