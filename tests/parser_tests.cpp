#include "pyc/lexer.hpp"
#include "pyc/parser.hpp"

#include <cassert>
#include <string>

using namespace pyc;

static std::vector<Stmt> parse(const std::string& source) {
    return Parser(Lexer::lex(source)).parse();
}

int main() {
    const auto lexed = Lexer::lex("def add(a, b):\n    return a + 1.5 * 1e10 # comment\n");
    assert(lexed.size() == 18);
    assert(lexed[0].kind == TokenKind::Def);
    assert(lexed[1].kind == TokenKind::Identifier);
    assert(lexed[1].value == "add");
    assert(lexed[8].kind == TokenKind::Newline);
    assert(lexed[9].kind == TokenKind::Indent);
    assert(lexed[13].kind == TokenKind::Float);
    assert(lexed[17].kind == TokenKind::Dedent);
    const auto comment = Lexer::lex("# comment\n");
    assert(comment.size() == 1);
    assert(comment[0].kind == TokenKind::Newline);

    const auto statements = parse("x = 10 + 5");
    assert(statements.size() == 1);
    assert(statements[0].kind == Stmt::Kind::Assign);
    assert(statements[0].name == "x");
    assert(statements[0].value.kind == Expr::Kind::Binary);
    const auto& addition = std::get<Expr::Binary>(statements[0].value.value);
    assert(addition.op == BinOp::Add);
    assert(addition.left->kind == Expr::Kind::Integer);
    assert(std::get<std::string>(addition.left->value) == "10");
    assert(addition.right->kind == Expr::Kind::Integer);
    assert(std::get<std::string>(addition.right->value) == "5");

    const auto precedence = parse("result = 2 + 3 * 4");
    const auto& result = std::get<Expr::Binary>(precedence[0].value.value);
    assert(result.op == BinOp::Add);
    assert(std::get<Expr::Binary>(result.right->value).op == BinOp::Mul);

    const auto multiple = parse("x = 1\ny = x * 2\n");
    assert(multiple.size() == 2);

    bool failed = false;
    try { parse("x = 1 y = 2"); } catch (const ParseError& error) {
        failed = error.message.find("newline or semicolon") != std::string::npos;
    }
    assert(failed);
}
