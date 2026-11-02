use super::{Lexer, Span, Token, TokenKind};

#[test]
fn lexes_python_like_source() {
    let tokens = Lexer::lex("def add(a, b):\n    return a + 1.5 * 1e10 # comment\n").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token {
                kind: TokenKind::Def,
                span: Span { start: 0, end: 3 }
            },
            Token {
                kind: TokenKind::Identifier("add".into()),
                span: Span { start: 4, end: 7 }
            },
            Token {
                kind: TokenKind::LeftParen,
                span: Span { start: 7, end: 8 }
            },
            Token {
                kind: TokenKind::Identifier("a".into()),
                span: Span { start: 8, end: 9 }
            },
            Token {
                kind: TokenKind::Comma,
                span: Span { start: 9, end: 10 }
            },
            Token {
                kind: TokenKind::Identifier("b".into()),
                span: Span { start: 11, end: 12 }
            },
            Token {
                kind: TokenKind::RightParen,
                span: Span { start: 12, end: 13 }
            },
            Token {
                kind: TokenKind::Colon,
                span: Span { start: 13, end: 14 }
            },
            Token {
                kind: TokenKind::Newline,
                span: Span { start: 14, end: 15 }
            },
            Token {
                kind: TokenKind::INDENT,
                span: Span { start: 15, end: 19 }
            },
            Token {
                kind: TokenKind::Return,
                span: Span { start: 19, end: 25 }
            },
            Token {
                kind: TokenKind::Identifier("a".into()),
                span: Span { start: 26, end: 27 }
            },
            Token {
                kind: TokenKind::Plus,
                span: Span { start: 28, end: 29 }
            },
            Token {
                kind: TokenKind::Float("1.5".into()),
                span: Span { start: 30, end: 33 }
            },
            Token {
                kind: TokenKind::Star,
                span: Span { start: 34, end: 35 }
            },
            Token {
                kind: TokenKind::Float("1e10".into()),
                span: Span { start: 36, end: 40 }
            },
            Token {
                kind: TokenKind::Newline,
                span: Span { start: 50, end: 51 }
            },
            Token {
                kind: TokenKind::DEDENT,
                span: Span { start: 51, end: 51 }
            },
        ]
    );
}

#[test]
fn reports_invalid_characters_and_unterminated_strings() {
    assert!(Lexer::lex("@").is_err());
    assert!(Lexer::lex("'unterminated").is_err());
}

#[test]
fn lexes_floor_division_assignment() {
    assert_eq!(
        Lexer::lex("value //= divisor").unwrap(),
        vec![
            Token {
                kind: TokenKind::Identifier("value".into()),
                span: Span { start: 0, end: 5 }
            },
            Token {
                kind: TokenKind::DoubleSlashEqual,
                span: Span { start: 6, end: 9 }
            },
            Token {
                kind: TokenKind::Identifier("divisor".into()),
                span: Span { start: 10, end: 17 }
            },
        ]
    );
}

#[test]
fn tracks_nested_indentation_and_ignores_blank_lines() {
    let tokens = Lexer::lex("if condition:\n    # comment\n    value = 1\n\nresult = 2").unwrap();
    assert_eq!(
        tokens.iter().map(|token| &token.kind).collect::<Vec<_>>(),
        vec![
            &TokenKind::If,
            &TokenKind::Identifier("condition".into()),
            &TokenKind::Colon,
            &TokenKind::Newline,
            &TokenKind::Newline,
            &TokenKind::INDENT,
            &TokenKind::Identifier("value".into()),
            &TokenKind::Equal,
            &TokenKind::Integer("1".into()),
            &TokenKind::Newline,
            &TokenKind::Newline,
            &TokenKind::DEDENT,
            &TokenKind::Identifier("result".into()),
            &TokenKind::Equal,
            &TokenKind::Integer("2".into()),
        ]
    );
}

#[test]
fn rejects_mismatched_indentation() {
    let error = Lexer::lex("if condition:\n    value = 1\n  result = 2").unwrap_err();
    assert!(error.message.contains("indentation"));
}
