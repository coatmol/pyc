use super::{BinOp, Expr, Parser, Stmt, UnaryOp};
use crate::lexer::Lexer;

fn parse(source: &str) -> Vec<Stmt> {
    let tokens = Lexer::lex(source).expect("source should lex");
    Parser::new(tokens).parse().expect("source should parse")
}

#[test]
fn parses_assignment_with_addition() {
    assert_eq!(
        parse("x = 10 + 5"),
        vec![Stmt::Assign {
            name: "x".to_string(),
            value: Expr::Binary {
                left: Box::new(Expr::Integer("10".to_string())),
                op: BinOp::Add,
                right: Box::new(Expr::Integer("5".to_string())),
            },
        }]
    );
}

#[test]
fn respects_operator_precedence() {
    assert_eq!(
        parse("result = 2 + 3 * 4"),
        vec![Stmt::Assign {
            name: "result".to_string(),
            value: Expr::Binary {
                left: Box::new(Expr::Integer("2".to_string())),
                op: BinOp::Add,
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::Integer("3".to_string())),
                    op: BinOp::Mul,
                    right: Box::new(Expr::Integer("4".to_string())),
                }),
            },
        }]
    );
}

#[test]
fn parses_parenthesized_and_unary_expressions() {
    assert_eq!(
        parse("value = -(1 + 2)"),
        vec![Stmt::Assign {
            name: "value".to_string(),
            value: Expr::Unary {
                op: UnaryOp::Minus,
                operand: Box::new(Expr::Binary {
                    left: Box::new(Expr::Integer("1".to_string())),
                    op: BinOp::Add,
                    right: Box::new(Expr::Integer("2".to_string())),
                }),
            },
        }]
    );
}

#[test]
fn parses_multiple_statements() {
    assert_eq!(
        parse("x = 1\ny = x * 2\n"),
        vec![
            Stmt::Assign {
                name: "x".to_string(),
                value: Expr::Integer("1".to_string()),
            },
            Stmt::Assign {
                name: "y".to_string(),
                value: Expr::Binary {
                    left: Box::new(Expr::Identifier("x".to_string())),
                    op: BinOp::Mul,
                    right: Box::new(Expr::Integer("2".to_string())),
                },
            },
        ]
    );
}

#[test]
fn reports_invalid_statement_separator() {
    let tokens = Lexer::lex("x = 1 y = 2").expect("source should lex");
    let error = Parser::new(tokens).parse().expect_err("source should fail");

    assert!(error.message.contains("newline or semicolon"));
}
