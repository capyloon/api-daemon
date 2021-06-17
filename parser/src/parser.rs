use std::fs::File;
use std::io::Read;
use std::path::Path;
use thiserror::Error;
use crate::ast::TypeExtraDecorator;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Parsing error: {}:{}:{}: {}", .0.path, .0.line, .0.col, .1)]
    ParseError(ParserState, String),
    #[error("Can't peek this token")]
    PeekError,
    #[error("End of stream reached")]
    Eof,
    #[error("IO Error")]
    Io(#[from] ::std::io::Error),
}

type Result<T> = ::std::result::Result<T, ParserError>;

#[derive(Debug, Clone)]
pub struct ParserState {
    path: String, // The file being parsed
    pos: usize,   // The current absolute position we are on
    line: usize,  // The current line we are on
    col: usize,   // The current column we are on
}

/// `ParseContext` wraps an input stream of chars to provide
/// error reporting.
pub struct ParserContext {
    pub state: ParserState,
    content: Vec<char>, // The input string that we wrap.
    pub decorator: Option<TypeExtraDecorator>,
}

impl ParserContext {
    pub fn from_str(source: &str, content: &str, decorator: Option<TypeExtraDecorator>) -> Result<ParserContext> {
        Ok(ParserContext {
            state: ParserState {
                path: source.to_owned(),
                pos: 0,
                line: 1,
                col: 1,
            },
            content: content.chars().collect(),
            decorator,
        })
    }

    pub fn from_file<P: AsRef<Path>>(path: P, decorator: Option<TypeExtraDecorator>) -> Result<ParserContext> {
        let mut file = File::open(path.as_ref())?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        ParserContext::from_str(path.as_ref().to_str().unwrap(), &content, decorator)
    }

    pub fn get_state(&self) -> ParserState {
        self.state.clone()
    }

    pub fn set_state(&mut self, state: ParserState) {
        self.state = state;
    }

    // Returns the next character is possible, without consuming it.
    fn peek(&mut self) -> Result<char> {
        if self.state.pos >= self.content.len() {
            Err(ParserError::Eof)
        } else {
            Ok(self.content[self.state.pos])
        }
    }

    // Returns the next character is possible, and consumes it.
    fn consume(&mut self) -> Result<char> {
        if self.state.pos >= self.content.len() {
            return Err(ParserError::Eof);
        }
        self.state.pos += 1;
        let c = self.content[self.state.pos - 1];
        if c == '\n' {
            self.state.line += 1;
            self.state.col = 1;
        } else {
            self.state.col += 1;
        }
        Ok(c)
    }

    /// Advances the position by going over whitespaces and comments.
    /// Comments are staring with `//` and end up at the end of the line.
    fn eat_whitespace_and_comments(&mut self) -> Result<()> {
        let mut in_comment = false;
        loop {
            let current = self.peek()?;
            // Detect if we are starting a new comment.
            if !in_comment && current == '/' {
                self.consume()?;
                if self.peek()? == '/' {
                    let _ = self.consume()?;
                    in_comment = true;
                    continue;
                }
            }
            // Detect if we are at the end of a comment.
            if in_comment && current == '\n' {
                let _ = self.consume()?;
                in_comment = false;
                continue;
            }
            // Check if we are still in whitespace.
            if current.is_whitespace() || in_comment {
                let _ = self.consume()?;
            } else {
                break;
            }
        }
        Ok(())
    }

    // Tries to get a token of a given kind, but resets the parser state
    // if that fails.
    pub fn peek_token(&mut self, kind: TokenKind) -> Result<Token> {
        let state = self.state.clone();
        match self.next_token(kind) {
            Ok(token) => Ok(token),
            Err(_) => {
                self.state = state;
                Err(ParserError::PeekError)
            }
        }
    }

    // Get the next token of a given kind, advancing the parser state.
    pub fn next_token(&mut self, kind: TokenKind) -> Result<Token> {
        macro_rules! parse_error {
            ($msg:expr) => {
                return Err(ParserError::ParseError(self.state.clone(), $msg));
            };
        }

        self.eat_whitespace_and_comments()?;

        match kind {
            TokenKind::Identifier => {
                let mut val = String::new();
                let mut current = self.consume()?;
                if !current.is_alphabetic() {
                    parse_error!(format!(
                        "Identifier must start with an alphabetic letter, but found: '{}'",
                        current
                    ));
                }

                // Read more until we hit a forbiddent character.
                loop {
                    val.push(current);
                    current = self.peek()?;
                    if !current.is_alphanumeric() && current != '_' {
                        break;
                    } else {
                        current = self.consume()?;
                    }
                }
                Ok(Token::Identifier(val))
            }
            TokenKind::Expected(expected) => {
                let s = expected.as_str();
                for c in s.chars() {
                    let current = self.consume()?;
                    if current != c {
                        parse_error!(format!("Expected {}, but found: '{}'", current, c));
                    }
                }
                Ok(Token::Empty)
            }
            TokenKind::LitteralString => {
                let current = self.consume()?;
                if current != '"' {
                    parse_error!(format!(
                        "Expected '\"' as a string delimiter, but found: '{}'",
                        current
                    ));
                }
                let mut val = String::new();
                loop {
                    let current = self.consume()?;
                    if current == '\\' {
                        // escape next char is it's " or \
                        let next = self.consume()?;
                        if next == '"' || next == '\\' {
                            val.push(next);
                        } else {
                            parse_error!(format!("Can't escape '{}' in string litterals", next));
                        }
                    } else if current != '"' {
                        val.push(current);
                    } else {
                        return Ok(Token::LitteralString(val));
                    }
                }
            }
            TokenKind::Annotation => {
                let mut current = self.peek()?;
                if current != '#' {
                    parse_error!(format!("Expected '#', but found '{}'", current));
                } else {
                    let _ = self.consume()?;
                }
                current = self.peek()?;
                if current != '[' {
                    parse_error!(format!("Expected '#', but found '{}'", current));
                } else {
                    let _ = self.consume()?;
                }
                let mut val = String::new();
                loop {
                    current = self.consume()?;
                    if current != ']' {
                        val.push(current);
                    } else {
                        break;
                    }
                }
                Ok(Token::Annotation(val))
            }
        }
    }
}

pub enum TokenKind {
    Identifier,       // Will match an identifier.
    Expected(String), // Will match a given string.
    LitteralString,   // A litteral string.
    Annotation,       // a #[] annotation.
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Empty,                  // A sentinel value when expecting a token without needing the value.
    Identifier(String),     // When you expect and identifier.
    LitteralString(String), // When you need to read a String.
    Annotation(String),     // The content of an annotation.
}

impl Token {
    pub fn as_str(&self) -> String {
        match *self {
            Token::Empty => "".to_owned(),
            Token::Identifier(ref s) | Token::LitteralString(ref s) | Token::Annotation(ref s) => {
                s.clone()
            }
        }
    }
}

#[test]
fn test_parser() {
    let content = r#"import "a_fi\"le.idl" // test

    type { 

    }

    #[annotation_example]
    service my_service { }
    "#;

    let mut ctxt = ParserContext::from_str("test", content, None).unwrap();
    //let mut ctxt = ParserContext::from_file("./test.idl").unwrap();

    let token = ctxt.next_token(TokenKind::Identifier).unwrap();
    assert_eq!(token, Token::Identifier("import".to_owned()));

    let mut token = ctxt.next_token(TokenKind::LitteralString).unwrap();
    assert_eq!(token, Token::LitteralString("a_fi\"le.idl".to_owned()));

    token = ctxt.next_token(TokenKind::Identifier).unwrap();
    assert_eq!(token, Token::Identifier("type".to_owned()));
    token = ctxt
        .next_token(TokenKind::Expected("{".to_owned()))
        .unwrap();
    assert_eq!(token, Token::Empty);
    token = ctxt
        .next_token(TokenKind::Expected("}".to_owned()))
        .unwrap();
    assert_eq!(token, Token::Empty);

    token = ctxt.next_token(TokenKind::Annotation).unwrap();
    assert_eq!(token, Token::Annotation("annotation_example".to_owned()));
    assert_eq!(ctxt.state.line, 7);
    assert_eq!(ctxt.state.col, 26);

    match ctxt.peek_token(TokenKind::Annotation).err().unwrap() {
        ParserError::PeekError => assert!(true),
        _ => assert!(false),
    }

    token = ctxt.next_token(TokenKind::Identifier).unwrap();
    assert_eq!(token, Token::Identifier("service".to_owned()));
    token = ctxt.next_token(TokenKind::Identifier).unwrap();
    assert_eq!(token, Token::Identifier("my_service".to_owned()));
    token = ctxt
        .next_token(TokenKind::Expected("{".to_owned()))
        .unwrap();
    assert_eq!(token, Token::Empty);
    token = ctxt
        .next_token(TokenKind::Expected("}".to_owned()))
        .unwrap();
    assert_eq!(token, Token::Empty);

    match ctxt.next_token(TokenKind::Identifier).err().unwrap() {
        ParserError::Eof => assert!(true),
        _ => assert!(false),
    }
}
