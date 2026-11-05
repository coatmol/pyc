#include "pyc/lexer.hpp"

#include <cctype>
#include <algorithm>
#include <stdexcept>
#include <unordered_map>

namespace pyc
{

    Lexer::Lexer(std::string source) : source_(std::move(source)) {}

    bool Lexer::has_current() const { return position_ < source_.size(); }
    char Lexer::current() const { return has_current() ? source_[position_] : '\0'; }
    char Lexer::bump() { return has_current() ? source_[position_++] : '\0'; }
    bool Lexer::matches(char expected)
    {
        if (current() == expected)
        {
            ++position_;
            return true;
        }
        return false;
    }

    Token Lexer::make_token(TokenKind kind, std::size_t start, std::string value) const
    {
        return {kind, std::move(value), {start, position_}};
    }

    [[noreturn]] void Lexer::fail(const std::string &message, std::size_t start) const
    {
        throw LexError{message, {start, std::max(position_, start + 1)}};
    }

    Token Lexer::read_identifier(std::size_t start)
    {
        while (has_current() && (std::isalnum(static_cast<unsigned char>(current())) || current() == '_'))
            ++position_;
        const auto value = source_.substr(start, position_ - start);
        static const std::unordered_map<std::string, TokenKind> keywords{
            {"False", TokenKind::False}, {"None", TokenKind::None}, {"True", TokenKind::True}, {"and", TokenKind::And}, {"as", TokenKind::As}, {"assert", TokenKind::Assert}, {"async", TokenKind::Async}, {"await", TokenKind::Await}, {"break", TokenKind::Break}, {"class", TokenKind::Class}, {"continue", TokenKind::Continue}, {"def", TokenKind::Def}, {"del", TokenKind::Del}, {"elif", TokenKind::Elif}, {"else", TokenKind::Else}, {"except", TokenKind::Except}, {"finally", TokenKind::Finally}, {"for", TokenKind::For}, {"from", TokenKind::From}, {"global", TokenKind::Global}, {"if", TokenKind::If}, {"import", TokenKind::Import}, {"in", TokenKind::In}, {"is", TokenKind::Is}, {"lambda", TokenKind::Lambda}, {"nonlocal", TokenKind::Nonlocal}, {"not", TokenKind::Not}, {"or", TokenKind::Or}, {"pass", TokenKind::Pass}, {"raise", TokenKind::Raise}, {"return", TokenKind::Return}, {"try", TokenKind::Try}, {"while", TokenKind::While}, {"with", TokenKind::With}, {"yield", TokenKind::Yield}};
        if (const auto it = keywords.find(value); it != keywords.end())
            return make_token(it->second, start);
        return make_token(TokenKind::Identifier, start, value);
    }

    Token Lexer::read_number(std::size_t start)
    {
        while (std::isdigit(static_cast<unsigned char>(current())))
            ++position_;
        bool floating = false;
        if (current() == '.' && position_ + 1 < source_.size() &&
            std::isdigit(static_cast<unsigned char>(source_[position_ + 1])))
        {
            floating = true;
            ++position_;
            while (std::isdigit(static_cast<unsigned char>(current())))
                ++position_;
        }
        if (current() == 'e' || current() == 'E')
        {
            floating = true;
            ++position_;
            matches('+');
            matches('-');
            const auto digits = position_;
            while (std::isdigit(static_cast<unsigned char>(current())))
                ++position_;
            if (digits == position_)
                return make_token(TokenKind::Identifier, start, source_.substr(start, position_ - start));
        }
        return make_token(floating ? TokenKind::Float : TokenKind::Integer, start,
                          source_.substr(start, position_ - start));
    }

    Token Lexer::read_string(std::size_t start, char quote)
    {
        std::string value;
        while (has_current())
        {
            const char c = bump();
            if (c == quote)
                return make_token(TokenKind::String, start, value);
            if (c == '\n' || c == '\r')
                fail("unterminated string literal", start);
            if (c == '\\')
            {
                if (!has_current())
                    fail("unterminated string literal", start);
                const char escaped = bump();
                value += escaped == 'n' ? '\n' : escaped == 'r' ? '\r'
                                             : escaped == 't'   ? '\t'
                                             : escaped == '\\'  ? '\\'
                                             : escaped == '\''  ? '\''
                                             : escaped == '"'   ? '"'
                                                                : escaped;
            }
            else
                value += c;
        }
        fail("unterminated string literal", start);
    }

    std::optional<Token> Lexer::next()
    {
        for (;;)
        {
            if (!pending_.empty())
            {
                auto token = pending_.back();
                pending_.pop_back();
                return token;
            }
            if (at_line_start_)
            {
                const auto indent_start = position_;
                std::size_t indent = 0;
                while (current() == ' ' || current() == '\t')
                {
                    if (bump() == '\t')
                        indent += 8 - indent % 8;
                    else
                        ++indent;
                }
                if (current() == '#')
                {
                    while (has_current() && current() != '\n')
                        ++position_;
                    continue;
                }
                if (current() != '\n' && has_current())
                {
                    at_line_start_ = false;
                    if (indent > indents_.back())
                    {
                        indents_.push_back(indent);
                        return Token{TokenKind::Indent, {}, {indent_start, position_}};
                    }
                    if (indent < indents_.back())
                    {
                        while (indent < indents_.back())
                        {
                            indents_.pop_back();
                            pending_.push_back({TokenKind::Dedent, {}, {position_, position_}});
                        }
                        if (indent != indents_.back())
                            fail("unindent does not match any outer indentation level", indent_start);
                        auto token = pending_.back();
                        pending_.pop_back();
                        return token;
                    }
                }
            }
            if (!has_current())
            {
                if (indents_.size() > 1)
                {
                    indents_.pop_back();
                    return Token{TokenKind::Dedent, {}, {position_, position_}};
                }
                return std::nullopt;
            }
            const auto start = position_;
            const char c = bump();
            if (c == ' ' || c == '\t' || c == '\r')
                continue;
            if (c == '\n')
            {
                if (paren_level_ > 0)
                    continue;
                at_line_start_ = true;
                return make_token(TokenKind::Newline, start);
            }
            if (c == '#')
            {
                while (has_current() && current() != '\n')
                    ++position_;
                continue;
            }
            if (std::isalpha(static_cast<unsigned char>(c)) || c == '_')
                return read_identifier(start);
            if (std::isdigit(static_cast<unsigned char>(c)))
                return read_number(start);
            if (c == '\'' || c == '"')
                return read_string(start, c);
            const auto simple = [&](TokenKind one, TokenKind equal)
            { return make_token(matches('=') ? equal : one, start); };
            switch (c)
            {
            case '+':
                return simple(TokenKind::Plus, TokenKind::PlusEqual);
            case '-':
                if (matches('='))
                    return make_token(TokenKind::MinusEqual, start);
                if (matches('>'))
                    return make_token(TokenKind::Arrow, start);
                return make_token(TokenKind::Minus, start);
            case '*':
                if (matches('*'))
                    return make_token(matches('=') ? TokenKind::DoubleStarEqual : TokenKind::DoubleStar, start);
                return simple(TokenKind::Star, TokenKind::StarEqual);
            case '/':
                if (matches('/'))
                    return make_token(matches('=') ? TokenKind::DoubleSlashEqual : TokenKind::DoubleSlash, start);
                return simple(TokenKind::Slash, TokenKind::SlashEqual);
            case '%':
                return simple(TokenKind::Percent, TokenKind::PercentEqual);
            case '=':
                return simple(TokenKind::Equal, TokenKind::EqualEqual);
            case '!':
                if (matches('='))
                    return make_token(TokenKind::NotEqual, start);
                fail("unexpected '!'", start);
            case '<':
                return simple(TokenKind::Less, TokenKind::LessEqual);
            case '>':
                return simple(TokenKind::Greater, TokenKind::GreaterEqual);
            case '(':
                ++paren_level_;
                return make_token(TokenKind::LeftParen, start);
            case ')':
                if (paren_level_ > 0)
                    --paren_level_;
                return make_token(TokenKind::RightParen, start);
            case '[':
                ++paren_level_;
                return make_token(TokenKind::LeftBracket, start);
            case ']':
                if (paren_level_ > 0)
                    --paren_level_;
                return make_token(TokenKind::RightBracket, start);
            case '{':
                ++paren_level_;
                return make_token(TokenKind::LeftBrace, start);
            case '}':
                if (paren_level_ > 0)
                    --paren_level_;
                return make_token(TokenKind::RightBrace, start);
            case ',':
                return make_token(TokenKind::Comma, start);
            case ':':
                return make_token(TokenKind::Colon, start);
            case '.':
                return make_token(TokenKind::Dot, start);
            case ';':
                return make_token(TokenKind::Semicolon, start);
            default:
                fail(std::string("unexpected character '") + c + "'", start);
            }
        }
    }

    std::vector<Token> Lexer::lex(const std::string &source)
    {
        Lexer lexer(source);
        std::vector<Token> tokens;
        while (auto token = lexer.next())
        {
            tokens.push_back(std::move(*token));
        }
        return tokens;
    }

}
