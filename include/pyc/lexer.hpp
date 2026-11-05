#pragma once

#include <cstddef>
#include <optional>
#include <string>
#include <vector>

namespace pyc
{

    struct Span
    {
        std::size_t start{};
        std::size_t end{};
    };

    enum class TokenKind
    {
        Identifier,
        Integer,
        Float,
        String,
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
        Indent,
        Dedent
    };

    struct Token
    {
        TokenKind kind;
        std::string value;
        Span span;
    };

    struct LexError
    {
        std::string message;
        Span span;
    };

    class Lexer
    {
    public:
        explicit Lexer(std::string source);
        static std::vector<Token> lex(const std::string &source);

    private:
        std::string source_;
        std::size_t position_{};
        std::size_t paren_level_{};
        std::vector<std::size_t> indents_{0};
        bool at_line_start_{true};
        std::vector<Token> pending_;

        char current() const;
        bool has_current() const;
        char bump();
        bool matches(char expected);
        Token make_token(TokenKind kind, std::size_t start, std::string value = {}) const;
        [[noreturn]] void fail(const std::string &message, std::size_t start) const;
        Token read_identifier(std::size_t start);
        Token read_number(std::size_t start);
        Token read_string(std::size_t start, char quote);
        std::optional<Token> next();
    };

}
