#pragma once

#include "pyc/lexer.hpp"

#include <memory>
#include <string>
#include <variant>
#include <vector>

namespace pyc
{

    enum class BinOp
    {
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
        Is
    };

    enum class UnaryOp
    {
        Plus,
        Minus,
        Not
    };

    struct Expr
    {
        struct Binary
        {
            std::shared_ptr<Expr> left;
            BinOp op;
            std::shared_ptr<Expr> right;
        };
        struct Unary
        {
            UnaryOp op;
            std::shared_ptr<Expr> operand;
        };

        using Value = std::variant<std::string, bool, std::monostate, Binary, Unary>;
        enum class Kind
        {
            Integer,
            Float,
            String,
            Identifier,
            Boolean,
            None,
            Binary,
            Unary
        };

        Kind kind;
        Value value;

        static Expr integer(std::string value);
        static Expr floating(std::string value);
        static Expr string(std::string value);
        static Expr identifier(std::string value);
        static Expr boolean(bool value);
        static Expr none();
        static Expr binary(Expr left, BinOp op, Expr right);
        static Expr unary(UnaryOp op, Expr operand);
    };

    struct Stmt
    {
        enum class Kind
        {
            Assign,
            Expression
        };
        Kind kind;
        std::string name;
        Expr value;

        static Stmt assign(std::string name, Expr value);
        static Stmt expression(Expr value);
    };

    struct ParseError
    {
        std::string message;
        Span span;
    };

    class Parser
    {
    public:
        explicit Parser(std::vector<Token> tokens);
        std::vector<Stmt> parse();
        Expr parse_expression();

    private:
        std::vector<Token> tokens_;
        std::size_t position_{};

        const Token *current() const;
        const Token *peek() const;
        Token advance();
        bool check(TokenKind kind) const;
        Stmt parse_statement();
        Expr parse_precedence(int minimum);
        Expr parse_prefix();
        std::pair<BinOp, int> binary_operator() const;
        Token expect(TokenKind kind);
        void skip_separators();
        [[noreturn]] void fail(const std::string &message) const;
    };

}
