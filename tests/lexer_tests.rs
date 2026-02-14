use stanu::lexer::{Lexer, Token};
use stanu::syntax_kind::SyntaxKind;

fn lex(input: &str) -> Vec<Token> {
    Lexer::new(input).tokenize()
}

fn kinds(input: &str) -> Vec<SyntaxKind> {
    lex(input).into_iter().map(|t| t.kind).collect()
}

// === Whitespace and newlines ===

#[test]
fn whitespace() {
    assert_eq!(kinds("   "), vec![SyntaxKind::WHITESPACE]);
}

#[test]
fn newline() {
    assert_eq!(kinds("\n"), vec![SyntaxKind::NEWLINE]);
}

#[test]
fn mixed_trivia() {
    assert_eq!(
        kinds("  \n\t "),
        vec![SyntaxKind::WHITESPACE, SyntaxKind::NEWLINE, SyntaxKind::WHITESPACE]
    );
}

// === Comments ===

#[test]
fn hash_comment() {
    assert_eq!(kinds("# comment"), vec![SyntaxKind::LINE_COMMENT]);
}

#[test]
fn double_slash_comment() {
    assert_eq!(kinds("// comment"), vec![SyntaxKind::LINE_COMMENT]);
}

#[test]
fn comment_stops_at_newline() {
    let tokens = lex("# comment\nident");
    assert_eq!(tokens[0].kind, SyntaxKind::LINE_COMMENT);
    assert_eq!(tokens[0].text, "# comment");
    assert_eq!(tokens[1].kind, SyntaxKind::NEWLINE);
    assert_eq!(tokens[2].kind, SyntaxKind::IDENT);
}

#[test]
fn block_comment() {
    assert_eq!(kinds("/* comment */"), vec![SyntaxKind::BLOCK_COMMENT]);
}

#[test]
fn nested_block_comment() {
    assert_eq!(kinds("/* outer /* inner */ end */"), vec![SyntaxKind::BLOCK_COMMENT]);
}

// === Numbers ===

#[test]
fn integer() {
    let tokens = lex("42");
    assert_eq!(tokens[0].kind, SyntaxKind::NUMBER);
    assert_eq!(tokens[0].text, "42");
}

#[test]
fn float() {
    let tokens = lex("3.14");
    assert_eq!(tokens[0].kind, SyntaxKind::NUMBER);
    assert_eq!(tokens[0].text, "3.14");
}

#[test]
fn scientific_notation() {
    let tokens = lex("1.5e10");
    assert_eq!(tokens[0].kind, SyntaxKind::NUMBER);
    assert_eq!(tokens[0].text, "1.5e10");
}

#[test]
fn scientific_with_sign() {
    let tokens = lex("2E-3");
    assert_eq!(tokens[0].kind, SyntaxKind::NUMBER);
    assert_eq!(tokens[0].text, "2E-3");
}

// === Identifiers and keywords ===

#[test]
fn identifier() {
    let tokens = lex("foo_bar");
    assert_eq!(tokens[0].kind, SyntaxKind::IDENT);
    assert_eq!(tokens[0].text, "foo_bar");
}

#[test]
fn identifier_with_hyphen() {
    let tokens = lex("my-resource");
    assert_eq!(tokens[0].kind, SyntaxKind::IDENT);
    assert_eq!(tokens[0].text, "my-resource");
}

#[test]
fn keywords() {
    assert_eq!(kinds("true"), vec![SyntaxKind::TRUE_KW]);
    assert_eq!(kinds("false"), vec![SyntaxKind::FALSE_KW]);
    assert_eq!(kinds("null"), vec![SyntaxKind::NULL_KW]);
    assert_eq!(kinds("for"), vec![SyntaxKind::FOR_KW]);
    assert_eq!(kinds("in"), vec![SyntaxKind::IN_KW]);
    assert_eq!(kinds("if"), vec![SyntaxKind::IF_KW]);
    assert_eq!(kinds("else"), vec![SyntaxKind::ELSE_KW]);
    assert_eq!(kinds("endif"), vec![SyntaxKind::ENDIF_KW]);
    assert_eq!(kinds("endfor"), vec![SyntaxKind::ENDFOR_KW]);
}

// === Operators ===

#[test]
fn arithmetic_operators() {
    assert_eq!(
        kinds("+ - * / %"),
        vec![
            SyntaxKind::PLUS,
            SyntaxKind::WHITESPACE,
            SyntaxKind::MINUS,
            SyntaxKind::WHITESPACE,
            SyntaxKind::STAR,
            SyntaxKind::WHITESPACE,
            SyntaxKind::SLASH,
            SyntaxKind::WHITESPACE,
            SyntaxKind::PERCENT,
        ]
    );
}

#[test]
fn comparison_operators() {
    assert_eq!(
        kinds("== != < <= > >="),
        vec![
            SyntaxKind::EQ_EQ,
            SyntaxKind::WHITESPACE,
            SyntaxKind::BANG_EQ,
            SyntaxKind::WHITESPACE,
            SyntaxKind::LT,
            SyntaxKind::WHITESPACE,
            SyntaxKind::LT_EQ,
            SyntaxKind::WHITESPACE,
            SyntaxKind::GT,
            SyntaxKind::WHITESPACE,
            SyntaxKind::GT_EQ,
        ]
    );
}

#[test]
fn logical_operators() {
    assert_eq!(
        kinds("&& || !"),
        vec![
            SyntaxKind::AMP_AMP,
            SyntaxKind::WHITESPACE,
            SyntaxKind::PIPE_PIPE,
            SyntaxKind::WHITESPACE,
            SyntaxKind::BANG,
        ]
    );
}

// === Punctuation ===

#[test]
fn punctuation() {
    assert_eq!(
        kinds("= => ( ) { } [ ] , . : ? ..."),
        vec![
            SyntaxKind::EQ,
            SyntaxKind::WHITESPACE,
            SyntaxKind::FAT_ARROW,
            SyntaxKind::WHITESPACE,
            SyntaxKind::PAREN_L,
            SyntaxKind::WHITESPACE,
            SyntaxKind::PAREN_R,
            SyntaxKind::WHITESPACE,
            SyntaxKind::BRACE_L,
            SyntaxKind::WHITESPACE,
            SyntaxKind::BRACE_R,
            SyntaxKind::WHITESPACE,
            SyntaxKind::BRACKET_L,
            SyntaxKind::WHITESPACE,
            SyntaxKind::BRACKET_R,
            SyntaxKind::WHITESPACE,
            SyntaxKind::COMMA,
            SyntaxKind::WHITESPACE,
            SyntaxKind::DOT,
            SyntaxKind::WHITESPACE,
            SyntaxKind::COLON,
            SyntaxKind::WHITESPACE,
            SyntaxKind::QUESTION,
            SyntaxKind::WHITESPACE,
            SyntaxKind::ELLIPSIS,
        ]
    );
}

// === Strings ===

#[test]
fn simple_string() {
    let tokens = lex(r#""hello""#);
    assert_eq!(
        kinds(r#""hello""#),
        vec![SyntaxKind::QUOTE, SyntaxKind::STRING_FRAGMENT, SyntaxKind::QUOTE]
    );
    assert_eq!(tokens[1].text, "hello");
}

#[test]
fn empty_string() {
    assert_eq!(
        kinds(r#""""#),
        vec![SyntaxKind::QUOTE, SyntaxKind::QUOTE]
    );
}

#[test]
fn string_with_escape() {
    assert_eq!(
        kinds(r#""hello\nworld""#),
        vec![
            SyntaxKind::QUOTE,
            SyntaxKind::STRING_FRAGMENT,
            SyntaxKind::ESCAPE_SEQUENCE,
            SyntaxKind::STRING_FRAGMENT,
            SyntaxKind::QUOTE,
        ]
    );
}

#[test]
fn string_with_interpolation() {
    let tokens = lex(r#""hello ${name}""#);
    assert_eq!(
        tokens.iter().map(|t| t.kind).collect::<Vec<_>>(),
        vec![
            SyntaxKind::QUOTE,
            SyntaxKind::STRING_FRAGMENT,
            SyntaxKind::DOLLAR_OPEN,
            SyntaxKind::IDENT,
            SyntaxKind::TEMPLATE_CLOSE,
            SyntaxKind::QUOTE,
        ]
    );
    assert_eq!(tokens[1].text, "hello ");
    assert_eq!(tokens[3].text, "name");
}

#[test]
fn string_with_directive() {
    assert_eq!(
        kinds(r#""%{if cond}yes%{endif}""#),
        vec![
            SyntaxKind::QUOTE,
            SyntaxKind::PERCENT_OPEN,
            SyntaxKind::IF_KW,
            SyntaxKind::WHITESPACE,
            SyntaxKind::IDENT,
            SyntaxKind::TEMPLATE_CLOSE,
            SyntaxKind::STRING_FRAGMENT,
            SyntaxKind::PERCENT_OPEN,
            SyntaxKind::ENDIF_KW,
            SyntaxKind::TEMPLATE_CLOSE,
            SyntaxKind::QUOTE,
        ]
    );
}

#[test]
fn nested_interpolation_with_braces() {
    // ${foo({a = 1})} - the inner {} should not close the interpolation
    let tokens = lex(r#""${foo({a = 1})}""#);
    let k: Vec<SyntaxKind> = tokens.iter().map(|t| t.kind).collect();
    assert_eq!(
        k,
        vec![
            SyntaxKind::QUOTE,
            SyntaxKind::DOLLAR_OPEN,
            SyntaxKind::IDENT,     // foo
            SyntaxKind::PAREN_L,
            SyntaxKind::BRACE_L,
            SyntaxKind::IDENT,     // a
            SyntaxKind::WHITESPACE,
            SyntaxKind::EQ,
            SyntaxKind::WHITESPACE,
            SyntaxKind::NUMBER,    // 1
            SyntaxKind::BRACE_R,
            SyntaxKind::PAREN_R,
            SyntaxKind::TEMPLATE_CLOSE,
            SyntaxKind::QUOTE,
        ]
    );
}

// === Heredocs ===

#[test]
fn simple_heredoc() {
    let input = "<<EOF\nhello world\nEOF";
    let tokens = lex(input);
    assert_eq!(tokens[0].kind, SyntaxKind::HEREDOC_OPEN);
    assert_eq!(tokens[0].text, "<<EOF\n");
    assert_eq!(tokens[1].kind, SyntaxKind::HEREDOC_CONTENT);
    assert_eq!(tokens[1].text, "hello world\n");
    assert_eq!(tokens[2].kind, SyntaxKind::HEREDOC_ANCHOR);
    assert_eq!(tokens[2].text, "EOF");
}

#[test]
fn indented_heredoc() {
    let input = "<<-EOF\n    hello\n    EOF";
    let tokens = lex(input);
    assert_eq!(tokens[0].kind, SyntaxKind::HEREDOC_OPEN);
    assert_eq!(tokens[1].kind, SyntaxKind::HEREDOC_CONTENT);
    assert_eq!(tokens[2].kind, SyntaxKind::HEREDOC_ANCHOR);
}

#[test]
fn heredoc_with_interpolation() {
    let input = "<<EOF\nhello ${name}\nEOF";
    let tokens = lex(input);
    assert_eq!(tokens[0].kind, SyntaxKind::HEREDOC_OPEN);
    assert_eq!(tokens[1].kind, SyntaxKind::HEREDOC_CONTENT);
    assert_eq!(tokens[1].text, "hello ");
    assert_eq!(tokens[2].kind, SyntaxKind::DOLLAR_OPEN);
    assert_eq!(tokens[3].kind, SyntaxKind::IDENT);
    assert_eq!(tokens[4].kind, SyntaxKind::TEMPLATE_CLOSE);
    assert_eq!(tokens[5].kind, SyntaxKind::HEREDOC_CONTENT);
    assert_eq!(tokens[6].kind, SyntaxKind::HEREDOC_ANCHOR);
}

// === Error tokens ===

#[test]
fn error_token_for_unknown_char() {
    let tokens = lex("@");
    assert_eq!(tokens[0].kind, SyntaxKind::ERROR_TOKEN);
    assert_eq!(tokens[0].text, "@");
}

// === Lossless round-trip ===

#[test]
fn lossless_roundtrip() {
    let inputs = [
        "x = 1\n",
        r#"name = "hello ${world}""#,
        "a = 1 + 2 * 3\n",
        "# comment\nfoo = bar\n",
        "resource \"aws\" \"x\" {\n  a = 1\n}\n",
    ];
    for input in inputs {
        let tokens = lex(input);
        let reconstructed: String = tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(reconstructed, input, "Round-trip failed for: {input:?}");
    }
}
