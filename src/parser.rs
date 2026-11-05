use std::fmt;

use super::lexer::{Span, Token, TokenKind};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Integer(String),
    Float(String),
    String(String),
    Identifier(String),
    Boolean(bool),
    None,
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    In,
    Is,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Assign { name: String, value: Expr },
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}..{}",
            self.message, self.span.start, self.span.end
        )
    }
}

impl std::error::Error for ParseError {}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position + 1)
    }

    fn advance(&mut self) -> Option<Token> {
        if self.position < self.tokens.len() {
            let token = self.tokens[self.position].clone();
            self.position += 1;
            Some(token)
        } else {
            None
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.current_token().is_some_and(|t| &t.kind == kind)
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        self.skip_separators();

        while self.current_token().is_some() {
            statements.push(self.parse_statement()?);
            if self.check(&TokenKind::Semicolon) {
                self.advance();
            } else if !self.check(&TokenKind::Newline)
                && !self.check(&TokenKind::DEDENT)
                && self.current_token().is_some()
            {
                return Err(self.error("expected a newline or semicolon"));
            }
            self.skip_separators();
        }

        Ok(statements)
    }

    pub fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_precedence(0)
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        if let Some(Token {
            kind: TokenKind::Identifier(name),
            ..
        }) = self.current_token()
        {
            if matches!(self.peek().map(|token| &token.kind), Some(TokenKind::Equal)) {
                let name = name.clone();
                self.advance();
                self.advance();
                return Ok(Stmt::Assign {
                    name,
                    value: self.parse_expression()?,
                });
            }
        }

        Ok(Stmt::Expr(self.parse_expression()?))
    }

    fn parse_precedence(&mut self, minimum: u8) -> Result<Expr, ParseError> {
        let mut left = self.parse_prefix()?;

        while let Some((operator, precedence)) = self.current_binary_operator() {
            if precedence < minimum {
                break;
            }
            self.advance();
            let right = self.parse_precedence(precedence + 1)?;
            left = Expr::Binary {
                left: Box::new(left),
                op: operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        let token = self
            .advance()
            .ok_or_else(|| self.error("expected an expression"))?;

        match token.kind {
            TokenKind::Integer(value) => Ok(Expr::Integer(value)),
            TokenKind::Float(value) => Ok(Expr::Float(value)),
            TokenKind::String(value) => Ok(Expr::String(value)),
            TokenKind::Identifier(value) => Ok(Expr::Identifier(value)),
            TokenKind::True => Ok(Expr::Boolean(true)),
            TokenKind::False => Ok(Expr::Boolean(false)),
            TokenKind::None => Ok(Expr::None),
            TokenKind::Plus => Ok(Expr::Unary {
                op: UnaryOp::Plus,
                operand: Box::new(self.parse_precedence(6)?),
            }),
            TokenKind::Minus => Ok(Expr::Unary {
                op: UnaryOp::Minus,
                operand: Box::new(self.parse_precedence(6)?),
            }),
            TokenKind::Not => Ok(Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(self.parse_precedence(2)?),
            }),
            TokenKind::LeftParen => {
                let expression = self.parse_expression()?;
                self.expect(TokenKind::RightParen)?;
                Ok(expression)
            }
            _ => Err(ParseError {
                message: "expected an expression".to_owned(),
                span: token.span,
            }),
        }
    }

    fn current_binary_operator(&self) -> Option<(BinOp, u8)> {
        let kind = &self.current_token()?.kind;
        Some(match kind {
            TokenKind::Or => (BinOp::Or, 1),
            TokenKind::And => (BinOp::And, 2),
            TokenKind::EqualEqual => (BinOp::Equal, 3),
            TokenKind::NotEqual => (BinOp::NotEqual, 3),
            TokenKind::Less => (BinOp::Less, 3),
            TokenKind::LessEqual => (BinOp::LessEqual, 3),
            TokenKind::Greater => (BinOp::Greater, 3),
            TokenKind::GreaterEqual => (BinOp::GreaterEqual, 3),
            TokenKind::In => (BinOp::In, 3),
            TokenKind::Is => (BinOp::Is, 3),
            TokenKind::Plus => (BinOp::Add, 4),
            TokenKind::Minus => (BinOp::Sub, 4),
            TokenKind::Star => (BinOp::Mul, 5),
            TokenKind::Slash => (BinOp::Div, 5),
            TokenKind::DoubleSlash => (BinOp::FloorDiv, 5),
            TokenKind::Percent => (BinOp::Mod, 5),
            _ => return None,
        })
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        if self.check(&kind) {
            return Ok(self.advance().expect("checked token exists"));
        }
        Err(self.error(format!("expected {kind:?}")))
    }

    fn skip_separators(&mut self) {
        while self.check(&TokenKind::Newline)
            || self.check(&TokenKind::INDENT)
            || self.check(&TokenKind::DEDENT)
        {
            self.advance();
        }
    }

    fn error(&self, message: impl Into<String>) -> ParseError {
        let span = self
            .current_token()
            .map_or(Span { start: 0, end: 0 }, |token| token.span);
        ParseError {
            message: message.into(),
            span,
        }
    }
}

#[cfg(test)]
mod tests;
