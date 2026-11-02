use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(String),
    Integer(String),
    Float(String),
    String(String),

    False,
    None,
    True,
    And,
    As,
    Assert,
    Async,
    Await,
    Break,
    Class,
    Continue,
    Def,
    Del,
    Elif,
    Else,
    Except,
    Finally,
    For,
    From,
    Global,
    If,
    Import,
    In,
    Is,
    Lambda,
    Nonlocal,
    Not,
    Or,
    Pass,
    Raise,
    Return,
    Try,
    While,
    With,
    Yield,

    Plus,
    Minus,
    Star,
    Slash,
    DoubleSlash,
    Percent,
    Equal,
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Arrow,
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    DoubleSlashEqual,
    PercentEqual,
    DoubleStar,
    DoubleStarEqual,

    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    Dot,
    Semicolon,

    Newline,
    INDENT,
    DEDENT,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for LexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}..{}",
            self.message, self.span.start, self.span.end
        )
    }
}

impl std::error::Error for LexError {}

pub struct Lexer<'source> {
    source: &'source str,
    position: usize,
    paren_level: usize,
    indents: Vec<usize>,
    at_line_start: bool,
    pending: Vec<Token>,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            source,
            position: 0,
            paren_level: 0,
            indents: vec![0],
            at_line_start: true,
            pending: Vec::new(),
        }
    }

    pub fn lex(source: &'source str) -> Result<Vec<Token>, LexError> {
        Self::new(source).collect()
    }

    fn current(&self) -> Option<char> {
        self.source[self.position..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let character = self.current()?;
        self.position += character.len_utf8();
        Some(character)
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.current() == Some(expected) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn token(&self, kind: TokenKind, start: usize) -> Token {
        Token {
            kind,
            span: Span::new(start, self.position),
        }
    }

    fn error(&self, message: impl Into<String>, start: usize) -> LexError {
        LexError {
            message: message.into(),
            span: Span::new(start, self.position.max(start + 1)),
        }
    }

    fn identifier(&mut self, start: usize) -> Token {
        while self.current().is_some_and(is_identifier_continue) {
            self.bump();
        }

        let value = &self.source[start..self.position];
        self.token(
            keyword(value).unwrap_or_else(|| TokenKind::Identifier(value.to_owned())),
            start,
        )
    }

    fn number(&mut self, start: usize) -> Token {
        while self
            .current()
            .is_some_and(|character| character.is_ascii_digit())
        {
            self.bump();
        }

        let mut kind = TokenKind::Integer(self.source[start..self.position].to_owned());
        if self.current() == Some('.')
            && self
                .source
                .get(self.position + 1..)
                .and_then(|rest| rest.chars().next())
                .is_some_and(|character| character.is_ascii_digit())
        {
            self.bump();
            while self
                .current()
                .is_some_and(|character| character.is_ascii_digit())
            {
                self.bump();
            }
            kind = TokenKind::Float(self.source[start..self.position].to_owned());
        }

        if self
            .current()
            .is_some_and(|character| character == 'e' || character == 'E')
        {
            self.bump();
            let _ = self.matches('+') || self.matches('-');
            let digits_start = self.position;
            while self
                .current()
                .is_some_and(|character| character.is_ascii_digit())
            {
                self.bump();
            }
            if digits_start == self.position {
                return self.token(
                    TokenKind::Identifier(self.source[start..self.position].to_owned()),
                    start,
                );
            }
            kind = TokenKind::Float(self.source[start..self.position].to_owned());
        }

        self.token(kind, start)
    }

    fn string(&mut self, start: usize, quote: char) -> Result<Token, LexError> {
        let mut value = String::new();
        while let Some(character) = self.bump() {
            match character {
                character if character == quote => {
                    return Ok(self.token(TokenKind::String(value), start));
                }
                '\\' => {
                    let escaped = self
                        .bump()
                        .ok_or_else(|| self.error("unterminated string literal", start))?;
                    value.push(match escaped {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '\\' => '\\',
                        '\'' => '\'',
                        '"' => '"',
                        other => other,
                    });
                }
                '\n' | '\r' => return Err(self.error("unterminated string literal", start)),
                other => value.push(other),
            }
        }
        Err(self.error("unterminated string literal", start))
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(token) = self.pending.pop() {
                return Some(Ok(token));
            }

            if self.at_line_start {
                let indent_start = self.position;
                let mut current_indent = 0;
                while let Some(character) = self.current() {
                    match character {
                        ' ' => {
                            current_indent += 1;
                            self.bump();
                        }
                        '\t' => {
                            current_indent += 8 - current_indent % 8;
                            self.bump();
                        }
                        _ => break,
                    }
                }

                if self.current().is_some_and(|character| character == '#') {
                    while self.current().is_some_and(|character| character != '\n') {
                        self.bump();
                    }
                    continue;
                }

                if self.current().is_some_and(|character| character != '\n') {
                    self.at_line_start = false;
                    if current_indent > *self.indents.last().unwrap() {
                        self.indents.push(current_indent);
                        return Some(Ok(Token {
                            kind: TokenKind::INDENT,
                            span: Span::new(indent_start, self.position),
                        }));
                    }
                    if current_indent < *self.indents.last().unwrap() {
                        let start_len = self.pending.len();
                        while current_indent < *self.indents.last().unwrap() {
                            self.indents.pop();
                            self.pending.push(Token {
                                kind: TokenKind::DEDENT,
                                span: Span::new(self.position, self.position),
                            });
                        }
                        // Reverse the newly pushed range so the deepest DEDENT comes out first
                        self.pending[start_len..].reverse();

                        if current_indent != *self.indents.last().unwrap() {
                            self.pending.clear();
                            return Some(Err(self.error(
                                "unindent does not match any outer indentation level",
                                indent_start,
                            )));
                        }
                        return self.pending.pop().map(|token| Ok(token));
                    }
                }
            }

            let start = self.position;
            let character = match self.bump() {
                Some(character) => character,
                None => {
                    if self.indents.len() > 1 {
                        self.indents.pop();
                        return Some(Ok(Token {
                            kind: TokenKind::DEDENT,
                            span: Span::new(self.position, self.position),
                        }));
                    }
                    return None;
                }
            };
            let result = match character {
                ' ' | '\t' | '\r' => continue,
                '\n' => {
                    if self.paren_level > 0 {
                        continue;
                    }

                    self.at_line_start = true;
                    Ok(self.token(TokenKind::Newline, start))
                }
                '#' => {
                    while self.current().is_some_and(|current| current != '\n') {
                        self.bump();
                    }
                    continue;
                }
                character if character.is_ascii_alphabetic() || character == '_' => {
                    Ok(self.identifier(start))
                }
                character if character.is_ascii_digit() => Ok(self.number(start)),
                '\'' | '"' => self.string(start, character),
                '+' => {
                    let kind = if self.matches('=') {
                        TokenKind::PlusEqual
                    } else {
                        TokenKind::Plus
                    };
                    Ok(self.token(kind, start))
                }
                '-' => {
                    let kind = if self.matches('=') {
                        TokenKind::MinusEqual
                    } else if self.matches('>') {
                        TokenKind::Arrow
                    } else {
                        TokenKind::Minus
                    };
                    Ok(self.token(kind, start))
                }
                '*' => {
                    let kind = if self.matches('*') {
                        if self.matches('=') {
                            TokenKind::DoubleStarEqual
                        } else {
                            TokenKind::DoubleStar
                        }
                    } else if self.matches('=') {
                        TokenKind::StarEqual
                    } else {
                        TokenKind::Star
                    };
                    Ok(self.token(kind, start))
                }
                '/' => {
                    let kind = if self.matches('/') {
                        if self.matches('=') {
                            TokenKind::DoubleSlashEqual
                        } else {
                            TokenKind::DoubleSlash
                        }
                    } else if self.matches('=') {
                        TokenKind::SlashEqual
                    } else {
                        TokenKind::Slash
                    };
                    Ok(self.token(kind, start))
                }
                '%' => {
                    let kind = if self.matches('=') {
                        TokenKind::PercentEqual
                    } else {
                        TokenKind::Percent
                    };
                    Ok(self.token(kind, start))
                }
                '=' => {
                    let kind = if self.matches('=') {
                        TokenKind::EqualEqual
                    } else {
                        TokenKind::Equal
                    };
                    Ok(self.token(kind, start))
                }
                '!' => {
                    if self.matches('=') {
                        Ok(self.token(TokenKind::NotEqual, start))
                    } else {
                        Err(self.error("unexpected '!'", start))
                    }
                }
                '<' => {
                    let kind = if self.matches('=') {
                        TokenKind::LessEqual
                    } else {
                        TokenKind::Less
                    };
                    Ok(self.token(kind, start))
                }
                '>' => {
                    let kind = if self.matches('=') {
                        TokenKind::GreaterEqual
                    } else {
                        TokenKind::Greater
                    };
                    Ok(self.token(kind, start))
                }
                '(' => {
                    self.paren_level += 1;
                    Ok(self.token(TokenKind::LeftParen, start))
                }
                ')' => {
                    if self.paren_level > 0 {
                        self.paren_level -= 1;
                    }
                    Ok(self.token(TokenKind::RightParen, start))
                }
                '[' => {
                    self.paren_level += 1;
                    Ok(self.token(TokenKind::LeftBracket, start))
                }
                ']' => {
                    if self.paren_level > 0 {
                        self.paren_level -= 1;
                    }
                    Ok(self.token(TokenKind::RightBracket, start))
                }
                '{' => {
                    self.paren_level += 1;
                    Ok(self.token(TokenKind::LeftBrace, start))
                }
                '}' => {
                    if self.paren_level > 0 {
                        self.paren_level -= 1;
                    }
                    Ok(self.token(TokenKind::RightBrace, start))
                }
                ',' => Ok(self.token(TokenKind::Comma, start)),
                ':' => Ok(self.token(TokenKind::Colon, start)),
                '.' => Ok(self.token(TokenKind::Dot, start)),
                ';' => Ok(self.token(TokenKind::Semicolon, start)),
                other => Err(self.error(format!("unexpected character '{other}'"), start)),
            };
            return Some(result);
        }
    }
}

fn is_identifier_continue(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}

fn keyword(value: &str) -> Option<TokenKind> {
    Some(match value {
        "False" => TokenKind::False,
        "None" => TokenKind::None,
        "True" => TokenKind::True,
        "and" => TokenKind::And,
        "as" => TokenKind::As,
        "assert" => TokenKind::Assert,
        "async" => TokenKind::Async,
        "await" => TokenKind::Await,
        "break" => TokenKind::Break,
        "class" => TokenKind::Class,
        "continue" => TokenKind::Continue,
        "def" => TokenKind::Def,
        "del" => TokenKind::Del,
        "elif" => TokenKind::Elif,
        "else" => TokenKind::Else,
        "except" => TokenKind::Except,
        "finally" => TokenKind::Finally,
        "for" => TokenKind::For,
        "from" => TokenKind::From,
        "global" => TokenKind::Global,
        "if" => TokenKind::If,
        "import" => TokenKind::Import,
        "in" => TokenKind::In,
        "is" => TokenKind::Is,
        "lambda" => TokenKind::Lambda,
        "nonlocal" => TokenKind::Nonlocal,
        "not" => TokenKind::Not,
        "or" => TokenKind::Or,
        "pass" => TokenKind::Pass,
        "raise" => TokenKind::Raise,
        "return" => TokenKind::Return,
        "try" => TokenKind::Try,
        "while" => TokenKind::While,
        "with" => TokenKind::With,
        "yield" => TokenKind::Yield,
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
