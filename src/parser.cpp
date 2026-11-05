#include "pyc/parser.hpp"

#include <stdexcept>
#include <utility>

namespace pyc
{

    Expr Expr::integer(std::string value) { return {Kind::Integer, std::move(value)}; }
    Expr Expr::floating(std::string value) { return {Kind::Float, std::move(value)}; }
    Expr Expr::string(std::string value) { return {Kind::String, std::move(value)}; }
    Expr Expr::identifier(std::string value) { return {Kind::Identifier, std::move(value)}; }
    Expr Expr::boolean(bool value) { return {Kind::Boolean, value}; }
    Expr Expr::none() { return {Kind::None, std::monostate{}}; }
    Expr Expr::binary(Expr left, BinOp op, Expr right)
    {
        return {Kind::Binary, Binary{std::make_shared<Expr>(std::move(left)), op, std::make_shared<Expr>(std::move(right))}};
    }
    Expr Expr::unary(UnaryOp op, Expr operand)
    {
        return {Kind::Unary, Unary{op, std::make_shared<Expr>(std::move(operand))}};
    }
    Stmt Stmt::assign(std::string name, Expr value) { return {Kind::Assign, std::move(name), std::move(value)}; }
    Stmt Stmt::expression(Expr value) { return {Kind::Expression, {}, std::move(value)}; }

    Parser::Parser(std::vector<Token> tokens) : tokens_(std::move(tokens)) {}
    const Token *Parser::current() const { return position_ < tokens_.size() ? &tokens_[position_] : nullptr; }
    const Token *Parser::peek() const { return position_ + 1 < tokens_.size() ? &tokens_[position_ + 1] : nullptr; }
    Token Parser::advance()
    {
        if (!current())
            fail("expected a token");
        return tokens_[position_++];
    }
    bool Parser::check(TokenKind kind) const { return current() && current()->kind == kind; }

    std::vector<Stmt> Parser::parse()
    {
        std::vector<Stmt> result;
        skip_separators();
        while (current())
        {
            result.push_back(parse_statement());
            if (check(TokenKind::Semicolon))
                advance();
            else if (current() && !check(TokenKind::Newline) && !check(TokenKind::Dedent))
                fail("expected a newline or semicolon");
            skip_separators();
        }
        return result;
    }

    Expr Parser::parse_expression() { return parse_precedence(0); }

    Stmt Parser::parse_statement()
    {
        if (current() && current()->kind == TokenKind::Identifier && peek() && peek()->kind == TokenKind::Equal)
        {
            const auto name = current()->value;
            advance();
            advance();
            return Stmt::assign(name, parse_expression());
        }
        return Stmt::expression(parse_expression());
    }

    Expr Parser::parse_precedence(int minimum)
    {
        auto left = parse_prefix();
        while (current())
        {
            auto [op, precedence] = binary_operator();
            if (precedence < minimum)
                break;
            advance();
            left = Expr::binary(std::move(left), op, parse_precedence(precedence + 1));
        }
        return left;
    }

    Expr Parser::parse_prefix()
    {
        const auto token = advance();
        switch (token.kind)
        {
        case TokenKind::Integer:
            return Expr::integer(token.value);
        case TokenKind::Float:
            return Expr::floating(token.value);
        case TokenKind::String:
            return Expr::string(token.value);
        case TokenKind::Identifier:
            return Expr::identifier(token.value);
        case TokenKind::True:
            return Expr::boolean(true);
        case TokenKind::False:
            return Expr::boolean(false);
        case TokenKind::None:
            return Expr::none();
        case TokenKind::Plus:
            return Expr::unary(UnaryOp::Plus, parse_precedence(6));
        case TokenKind::Minus:
            return Expr::unary(UnaryOp::Minus, parse_precedence(6));
        case TokenKind::Not:
            return Expr::unary(UnaryOp::Not, parse_precedence(2));
        case TokenKind::LeftParen:
        {
            auto expression = parse_expression();
            expect(TokenKind::RightParen);
            return expression;
        }
        default:
            throw ParseError{"expected an expression", token.span};
        }
    }

    std::pair<BinOp, int> Parser::binary_operator() const
    {
        if (!current())
            return {BinOp::Add, -1};
        switch (current()->kind)
        {
        case TokenKind::Or:
            return {BinOp::Or, 1};
        case TokenKind::And:
            return {BinOp::And, 2};
        case TokenKind::EqualEqual:
            return {BinOp::Equal, 3};
        case TokenKind::NotEqual:
            return {BinOp::NotEqual, 3};
        case TokenKind::Less:
            return {BinOp::Less, 3};
        case TokenKind::LessEqual:
            return {BinOp::LessEqual, 3};
        case TokenKind::Greater:
            return {BinOp::Greater, 3};
        case TokenKind::GreaterEqual:
            return {BinOp::GreaterEqual, 3};
        case TokenKind::In:
            return {BinOp::In, 3};
        case TokenKind::Is:
            return {BinOp::Is, 3};
        case TokenKind::Plus:
            return {BinOp::Add, 4};
        case TokenKind::Minus:
            return {BinOp::Sub, 4};
        case TokenKind::Star:
            return {BinOp::Mul, 5};
        case TokenKind::Slash:
            return {BinOp::Div, 5};
        case TokenKind::DoubleSlash:
            return {BinOp::FloorDiv, 5};
        case TokenKind::Percent:
            return {BinOp::Mod, 5};
        default:
            return {BinOp::Add, -1};
        }
    }

    Token Parser::expect(TokenKind kind)
    {
        if (!check(kind))
            fail("unexpected token");
        return advance();
    }
    void Parser::skip_separators()
    {
        while (check(TokenKind::Newline) || check(TokenKind::Indent) || check(TokenKind::Dedent))
            advance();
    }
    [[noreturn]] void Parser::fail(const std::string &message) const
    {
        throw ParseError{message, current() ? current()->span : Span{0, 0}};
    }

}
