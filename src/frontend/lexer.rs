#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Identifier(String),
    Integer(i64),
    Float(String),
    StringLiteral(String),
    CharLiteral(char),
    Semicolon,
    Comma,
    Colon,
    ColonColon,
    Dot,
    DoubleDot,
    Equal,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Plus,
    Minus,
    Star,
    Slash,
    Ampersand,
    Question,
    At,
    Arrow,
    FatArrow,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    EndOfFile,
}

#[derive(Debug, Clone)]
pub struct Lexer<'source> {
    source: &'source str,
    position: usize,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str) -> Self {
        Self { source, position: 0 }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let Some(ch) = self.peek_char() else {
            return Token::EndOfFile;
        };

        if ch.is_ascii_digit() {
            return self.lex_number();
        }

        if is_identifier_start(ch) {
            return self.lex_identifier();
        }

        if ch == '\'' {
            return self.lex_char();
        }

        if ch == '"' {
            return self.lex_string();
        }

        if self.match_str("=>") {
            self.position += 2;
            return Token::FatArrow;
        }

        if self.match_str("->") {
            self.position += 2;
            return Token::Arrow;
        }

        if self.match_str("::") {
            self.position += 2;
            return Token::ColonColon;
        }

        if self.match_str("==") {
            self.position += 2;
            return Token::EqualEqual;
        }

        if self.match_str("!=") {
            self.position += 2;
            return Token::BangEqual;
        }

        if self.match_str("<=") {
            self.position += 2;
            return Token::LessEqual;
        }

        if self.match_str(">=") {
            self.position += 2;
            return Token::GreaterEqual;
        }

        if self.match_str("..") {
            self.position += 2;
            return Token::DoubleDot;
        }

        self.position += ch.len_utf8();
        match ch {
            ';' => Token::Semicolon,
            ',' => Token::Comma,
            ':' => Token::Colon,
            '.' => Token::Dot,
            '=' => Token::Equal,
            '<' => Token::Less,
            '>' => Token::Greater,
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '&' => Token::Ampersand,
            '?' => Token::Question,
            '@' => Token::At,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            _ => Token::EndOfFile,
        }
    }

    fn lex_number(&mut self) -> Token {
        let start = self.position;
        let mut saw_decimal_point = false;

        while let Some(ch) = self.source[self.position..].chars().next() {
            if ch.is_ascii_digit() {
                self.position += ch.len_utf8();
            } else if ch == '.' && !saw_decimal_point && self.peek_next_char() != Some('.') {
                saw_decimal_point = true;
                self.position += ch.len_utf8();
            } else {
                break;
            }
        }

        let text = &self.source[start..self.position];
        if saw_decimal_point {
            Token::Float(text.to_string())
        } else {
            let value = text.parse::<i64>().unwrap_or(0);
            Token::Integer(value)
        }
    }

    fn lex_string(&mut self) -> Token {
        self.position += 1;
        let start = self.position;

        while let Some(ch) = self.peek_char() {
            if ch == '"' {
                let text = &self.source[start..self.position];
                self.position += 1;
                return Token::StringLiteral(text.to_string());
            }

            self.position += ch.len_utf8();
        }

        Token::StringLiteral(self.source[start..self.position].to_string())
    }

    fn lex_char(&mut self) -> Token {
        self.position += 1;
        let ch = self.peek_char().unwrap_or('\0');
        if ch != '\0' {
            self.position += ch.len_utf8();
        }
        if self.peek_char() == Some('\'') {
            self.position += 1;
        }
        Token::CharLiteral(ch)
    }

    fn lex_identifier(&mut self) -> Token {
        let start = self.position;
        while let Some(ch) = self.source[self.position..].chars().next() {
            if is_identifier_continue(ch) {
                self.position += ch.len_utf8();
            } else {
                break;
            }
        }

        let text = &self.source[start..self.position];
        Token::Identifier(text.to_string())
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.position += ch.len_utf8();
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.position..].chars().next()
    }

    fn peek_next_char(&self) -> Option<char> {
        self.source[self.position..].chars().nth(1)
    }

    fn match_str(&self, pattern: &str) -> bool {
        self.source[self.position..].starts_with(pattern)
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}
