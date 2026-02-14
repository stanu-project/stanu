use expect_test::{expect, Expect};
use stanu::parse_file;
use stanu::syntax_kind::SyntaxNode;

fn check(input: &str, expected: Expect) {
    let (green, errors) = parse_file(input);
    let node = SyntaxNode::new_root(green);
    let mut output = format!("{node:#?}");
    if !errors.is_empty() {
        output.push_str("\nErrors:\n");
        for err in &errors {
            output.push_str(&format!("  {err}\n"));
        }
    }
    expected.assert_eq(&output);
}

// === Attributes ===

#[test]
fn simple_attribute() {
    check(
        "x = 1\n",
        expect![[r#"
            SOURCE_FILE@0..6
              BODY@0..6
                ATTRIBUTE@0..6
                  IDENT@0..1 "x"
                  WHITESPACE@1..2 " "
                  EQ@2..3 "="
                  WHITESPACE@3..4 " "
                  LITERAL_EXPR@4..5
                    NUMBER@4..5 "1"
                  NEWLINE@5..6 "\n"
        "#]],
    );
}

#[test]
fn string_attribute() {
    check(
        r#"name = "hello"
"#,
        expect![[r#"
            SOURCE_FILE@0..15
              BODY@0..15
                ATTRIBUTE@0..15
                  IDENT@0..4 "name"
                  WHITESPACE@4..5 " "
                  EQ@5..6 "="
                  WHITESPACE@6..7 " "
                  STRING_EXPR@7..14
                    QUOTE@7..8 "\""
                    STRING_FRAGMENT@8..13 "hello"
                    QUOTE@13..14 "\""
                  NEWLINE@14..15 "\n"
        "#]],
    );
}

#[test]
fn bool_attribute() {
    check(
        "enabled = true\n",
        expect![[r#"
            SOURCE_FILE@0..15
              BODY@0..15
                ATTRIBUTE@0..15
                  IDENT@0..7 "enabled"
                  WHITESPACE@7..8 " "
                  EQ@8..9 "="
                  WHITESPACE@9..10 " "
                  LITERAL_EXPR@10..14
                    TRUE_KW@10..14 "true"
                  NEWLINE@14..15 "\n"
        "#]],
    );
}

// === Blocks ===

#[test]
fn empty_block() {
    check(
        "resource {\n}\n",
        expect![[r#"
            SOURCE_FILE@0..13
              BODY@0..13
                BLOCK@0..13
                  IDENT@0..8 "resource"
                  WHITESPACE@8..9 " "
                  BRACE_L@9..10 "{"
                  NEWLINE@10..11 "\n"
                  BODY@11..11
                  BRACE_R@11..12 "}"
                  NEWLINE@12..13 "\n"
        "#]],
    );
}

#[test]
fn block_with_string_labels() {
    check(
        "resource \"aws_instance\" \"web\" {\n}\n",
        expect![[r#"
            SOURCE_FILE@0..34
              BODY@0..34
                BLOCK@0..34
                  IDENT@0..8 "resource"
                  WHITESPACE@8..9 " "
                  BLOCK_LABEL@9..23
                    STRING_EXPR@9..23
                      QUOTE@9..10 "\""
                      STRING_FRAGMENT@10..22 "aws_instance"
                      QUOTE@22..23 "\""
                  WHITESPACE@23..24 " "
                  BLOCK_LABEL@24..29
                    STRING_EXPR@24..29
                      QUOTE@24..25 "\""
                      STRING_FRAGMENT@25..28 "web"
                      QUOTE@28..29 "\""
                  WHITESPACE@29..30 " "
                  BRACE_L@30..31 "{"
                  NEWLINE@31..32 "\n"
                  BODY@32..32
                  BRACE_R@32..33 "}"
                  NEWLINE@33..34 "\n"
        "#]],
    );
}

#[test]
fn block_with_body() {
    check(
        "resource {\n  x = 1\n}\n",
        expect![[r#"
            SOURCE_FILE@0..21
              BODY@0..21
                BLOCK@0..21
                  IDENT@0..8 "resource"
                  WHITESPACE@8..9 " "
                  BRACE_L@9..10 "{"
                  NEWLINE@10..11 "\n"
                  BODY@11..19
                    WHITESPACE@11..13 "  "
                    ATTRIBUTE@13..19
                      IDENT@13..14 "x"
                      WHITESPACE@14..15 " "
                      EQ@15..16 "="
                      WHITESPACE@16..17 " "
                      LITERAL_EXPR@17..18
                        NUMBER@17..18 "1"
                      NEWLINE@18..19 "\n"
                  BRACE_R@19..20 "}"
                  NEWLINE@20..21 "\n"
        "#]],
    );
}

// === Expressions ===

#[test]
fn binary_expr() {
    check(
        "x = 1 + 2\n",
        expect![[r#"
            SOURCE_FILE@0..10
              BODY@0..10
                ATTRIBUTE@0..10
                  IDENT@0..1 "x"
                  WHITESPACE@1..2 " "
                  EQ@2..3 "="
                  WHITESPACE@3..4 " "
                  BINARY_EXPR@4..10
                    LITERAL_EXPR@4..5
                      NUMBER@4..5 "1"
                    WHITESPACE@5..6 " "
                    PLUS@6..7 "+"
                    WHITESPACE@7..8 " "
                    LITERAL_EXPR@8..9
                      NUMBER@8..9 "2"
                    NEWLINE@9..10 "\n"
        "#]],
    );
}

#[test]
fn binary_expr_precedence() {
    check(
        "x = 1 + 2 * 3\n",
        expect![[r#"
            SOURCE_FILE@0..14
              BODY@0..14
                ATTRIBUTE@0..14
                  IDENT@0..1 "x"
                  WHITESPACE@1..2 " "
                  EQ@2..3 "="
                  WHITESPACE@3..4 " "
                  BINARY_EXPR@4..14
                    LITERAL_EXPR@4..5
                      NUMBER@4..5 "1"
                    WHITESPACE@5..6 " "
                    PLUS@6..7 "+"
                    WHITESPACE@7..8 " "
                    BINARY_EXPR@8..14
                      LITERAL_EXPR@8..9
                        NUMBER@8..9 "2"
                      WHITESPACE@9..10 " "
                      STAR@10..11 "*"
                      WHITESPACE@11..12 " "
                      LITERAL_EXPR@12..13
                        NUMBER@12..13 "3"
                      NEWLINE@13..14 "\n"
        "#]],
    );
}

#[test]
fn unary_expr() {
    check(
        "x = -5\n",
        expect![[r#"
            SOURCE_FILE@0..7
              BODY@0..7
                ATTRIBUTE@0..7
                  IDENT@0..1 "x"
                  WHITESPACE@1..2 " "
                  EQ@2..3 "="
                  WHITESPACE@3..4 " "
                  UNARY_EXPR@4..7
                    MINUS@4..5 "-"
                    LITERAL_EXPR@5..6
                      NUMBER@5..6 "5"
                    NEWLINE@6..7 "\n"
        "#]],
    );
}

#[test]
fn conditional_expr() {
    check(
        "x = a ? b : c\n",
        expect![[r#"
            SOURCE_FILE@0..14
              BODY@0..14
                ATTRIBUTE@0..14
                  IDENT@0..1 "x"
                  WHITESPACE@1..2 " "
                  EQ@2..3 "="
                  WHITESPACE@3..4 " "
                  CONDITIONAL_EXPR@4..14
                    VARIABLE_EXPR@4..5
                      IDENT@4..5 "a"
                    WHITESPACE@5..6 " "
                    QUESTION@6..7 "?"
                    WHITESPACE@7..8 " "
                    VARIABLE_EXPR@8..9
                      IDENT@8..9 "b"
                    WHITESPACE@9..10 " "
                    COLON@10..11 ":"
                    WHITESPACE@11..12 " "
                    VARIABLE_EXPR@12..13
                      IDENT@12..13 "c"
                    NEWLINE@13..14 "\n"
        "#]],
    );
}

#[test]
fn function_call() {
    check(
        "x = length(list)\n",
        expect![[r#"
            SOURCE_FILE@0..17
              BODY@0..17
                ATTRIBUTE@0..17
                  IDENT@0..1 "x"
                  WHITESPACE@1..2 " "
                  EQ@2..3 "="
                  WHITESPACE@3..4 " "
                  FUNCTION_CALL@4..16
                    IDENT@4..10 "length"
                    PAREN_L@10..11 "("
                    ARG_LIST@11..15
                      VARIABLE_EXPR@11..15
                        IDENT@11..15 "list"
                    PAREN_R@15..16 ")"
                  NEWLINE@16..17 "\n"
        "#]],
    );
}

#[test]
fn function_call_multiple_args() {
    check(
        "x = join(\",\", list)\n",
        expect![[r#"
            SOURCE_FILE@0..20
              BODY@0..20
                ATTRIBUTE@0..20
                  IDENT@0..1 "x"
                  WHITESPACE@1..2 " "
                  EQ@2..3 "="
                  WHITESPACE@3..4 " "
                  FUNCTION_CALL@4..19
                    IDENT@4..8 "join"
                    PAREN_L@8..9 "("
                    ARG_LIST@9..18
                      STRING_EXPR@9..12
                        QUOTE@9..10 "\""
                        STRING_FRAGMENT@10..11 ","
                        QUOTE@11..12 "\""
                      COMMA@12..13 ","
                      WHITESPACE@13..14 " "
                      VARIABLE_EXPR@14..18
                        IDENT@14..18 "list"
                    PAREN_R@18..19 ")"
                  NEWLINE@19..20 "\n"
        "#]],
    );
}

#[test]
fn tuple_expr() {
    check(
        "x = [1, 2, 3]\n",
        expect![[r#"
            SOURCE_FILE@0..14
              BODY@0..14
                ATTRIBUTE@0..14
                  IDENT@0..1 "x"
                  WHITESPACE@1..2 " "
                  EQ@2..3 "="
                  WHITESPACE@3..4 " "
                  TUPLE_EXPR@4..13
                    BRACKET_L@4..5 "["
                    LITERAL_EXPR@5..6
                      NUMBER@5..6 "1"
                    COMMA@6..7 ","
                    WHITESPACE@7..8 " "
                    LITERAL_EXPR@8..9
                      NUMBER@8..9 "2"
                    COMMA@9..10 ","
                    WHITESPACE@10..11 " "
                    LITERAL_EXPR@11..12
                      NUMBER@11..12 "3"
                    BRACKET_R@12..13 "]"
                  NEWLINE@13..14 "\n"
        "#]],
    );
}

#[test]
fn object_expr() {
    check(
        "x = {a = 1}\n",
        expect![[r#"
            SOURCE_FILE@0..12
              BODY@0..12
                ATTRIBUTE@0..12
                  IDENT@0..1 "x"
                  WHITESPACE@1..2 " "
                  EQ@2..3 "="
                  WHITESPACE@3..4 " "
                  OBJECT_EXPR@4..11
                    BRACE_L@4..5 "{"
                    OBJECT_ELEM@5..10
                      VARIABLE_EXPR@5..6
                        IDENT@5..6 "a"
                      WHITESPACE@6..7 " "
                      EQ@7..8 "="
                      WHITESPACE@8..9 " "
                      LITERAL_EXPR@9..10
                        NUMBER@9..10 "1"
                    BRACE_R@10..11 "}"
                  NEWLINE@11..12 "\n"
        "#]],
    );
}

#[test]
fn attr_access() {
    check(
        "x = a.b\n",
        expect![[r#"
            SOURCE_FILE@0..8
              BODY@0..8
                ATTRIBUTE@0..8
                  IDENT@0..1 "x"
                  WHITESPACE@1..2 " "
                  EQ@2..3 "="
                  WHITESPACE@3..4 " "
                  ATTR_ACCESS_EXPR@4..7
                    VARIABLE_EXPR@4..5
                      IDENT@4..5 "a"
                    DOT@5..6 "."
                    IDENT@6..7 "b"
                  NEWLINE@7..8 "\n"
        "#]],
    );
}

#[test]
fn index_expr() {
    check(
        "x = a[0]\n",
        expect![[r#"
            SOURCE_FILE@0..9
              BODY@0..9
                ATTRIBUTE@0..9
                  IDENT@0..1 "x"
                  WHITESPACE@1..2 " "
                  EQ@2..3 "="
                  WHITESPACE@3..4 " "
                  INDEX_EXPR@4..8
                    VARIABLE_EXPR@4..5
                      IDENT@4..5 "a"
                    BRACKET_L@5..6 "["
                    LITERAL_EXPR@6..7
                      NUMBER@6..7 "0"
                    BRACKET_R@7..8 "]"
                  NEWLINE@8..9 "\n"
        "#]],
    );
}

#[test]
fn paren_expr() {
    check(
        "x = (1 + 2)\n",
        expect![[r#"
            SOURCE_FILE@0..12
              BODY@0..12
                ATTRIBUTE@0..12
                  IDENT@0..1 "x"
                  WHITESPACE@1..2 " "
                  EQ@2..3 "="
                  WHITESPACE@3..4 " "
                  PAREN_EXPR@4..11
                    PAREN_L@4..5 "("
                    BINARY_EXPR@5..10
                      LITERAL_EXPR@5..6
                        NUMBER@5..6 "1"
                      WHITESPACE@6..7 " "
                      PLUS@7..8 "+"
                      WHITESPACE@8..9 " "
                      LITERAL_EXPR@9..10
                        NUMBER@9..10 "2"
                    PAREN_R@10..11 ")"
                  NEWLINE@11..12 "\n"
        "#]],
    );
}

#[test]
fn for_tuple_expr() {
    check(
        "x = [for s in list : upper(s)]\n",
        expect![[r#"
            SOURCE_FILE@0..31
              BODY@0..31
                ATTRIBUTE@0..31
                  IDENT@0..1 "x"
                  WHITESPACE@1..2 " "
                  EQ@2..3 "="
                  WHITESPACE@3..4 " "
                  FOR_TUPLE_EXPR@4..30
                    BRACKET_L@4..5 "["
                    FOR_INTRO@5..20
                      FOR_KW@5..8 "for"
                      WHITESPACE@8..9 " "
                      IDENT@9..10 "s"
                      WHITESPACE@10..11 " "
                      IN_KW@11..13 "in"
                      WHITESPACE@13..14 " "
                      VARIABLE_EXPR@14..18
                        IDENT@14..18 "list"
                      WHITESPACE@18..19 " "
                      COLON@19..20 ":"
                    WHITESPACE@20..21 " "
                    FUNCTION_CALL@21..29
                      IDENT@21..26 "upper"
                      PAREN_L@26..27 "("
                      ARG_LIST@27..28
                        VARIABLE_EXPR@27..28
                          IDENT@27..28 "s"
                      PAREN_R@28..29 ")"
                    BRACKET_R@29..30 "]"
                  NEWLINE@30..31 "\n"
        "#]],
    );
}

#[test]
fn for_object_expr() {
    check(
        "x = {for k, v in map : k => v}\n",
        expect![[r#"
            SOURCE_FILE@0..31
              BODY@0..31
                ATTRIBUTE@0..31
                  IDENT@0..1 "x"
                  WHITESPACE@1..2 " "
                  EQ@2..3 "="
                  WHITESPACE@3..4 " "
                  FOR_OBJECT_EXPR@4..30
                    BRACE_L@4..5 "{"
                    FOR_INTRO@5..22
                      FOR_KW@5..8 "for"
                      WHITESPACE@8..9 " "
                      IDENT@9..10 "k"
                      COMMA@10..11 ","
                      WHITESPACE@11..12 " "
                      IDENT@12..13 "v"
                      WHITESPACE@13..14 " "
                      IN_KW@14..16 "in"
                      WHITESPACE@16..17 " "
                      VARIABLE_EXPR@17..20
                        IDENT@17..20 "map"
                      WHITESPACE@20..21 " "
                      COLON@21..22 ":"
                    WHITESPACE@22..23 " "
                    VARIABLE_EXPR@23..24
                      IDENT@23..24 "k"
                    WHITESPACE@24..25 " "
                    FAT_ARROW@25..27 "=>"
                    WHITESPACE@27..28 " "
                    VARIABLE_EXPR@28..29
                      IDENT@28..29 "v"
                    BRACE_R@29..30 "}"
                  NEWLINE@30..31 "\n"
        "#]],
    );
}

// === String with interpolation ===

#[test]
fn string_interpolation() {
    check(
        r#"x = "hello ${name}"
"#,
        expect![[r#"
            SOURCE_FILE@0..20
              BODY@0..20
                ATTRIBUTE@0..20
                  IDENT@0..1 "x"
                  WHITESPACE@1..2 " "
                  EQ@2..3 "="
                  WHITESPACE@3..4 " "
                  STRING_EXPR@4..19
                    QUOTE@4..5 "\""
                    STRING_FRAGMENT@5..11 "hello "
                    TEMPLATE_INTERPOLATION@11..18
                      DOLLAR_OPEN@11..13 "${"
                      VARIABLE_EXPR@13..17
                        IDENT@13..17 "name"
                      TEMPLATE_CLOSE@17..18 "}"
                    QUOTE@18..19 "\""
                  NEWLINE@19..20 "\n"
        "#]],
    );
}

// === Splat ===

#[test]
fn attr_splat() {
    check(
        "x = items.*.name\n",
        expect![[r#"
            SOURCE_FILE@0..17
              BODY@0..17
                ATTRIBUTE@0..17
                  IDENT@0..1 "x"
                  WHITESPACE@1..2 " "
                  EQ@2..3 "="
                  WHITESPACE@3..4 " "
                  ATTR_SPLAT_EXPR@4..17
                    VARIABLE_EXPR@4..9
                      IDENT@4..9 "items"
                    DOT@9..10 "."
                    STAR@10..11 "*"
                    SPLAT_BODY@11..17
                      ATTR_ACCESS_EXPR@11..16
                        DOT@11..12 "."
                        IDENT@12..16 "name"
                      NEWLINE@16..17 "\n"
        "#]],
    );
}

// === Error recovery ===

#[test]
fn error_recovery_continues_parsing() {
    let input = "x = 1\n!!!\ny = 2\n";
    let (green, errors) = parse_file(input);
    let node = SyntaxNode::new_root(green);
    // Should have errors but still parse y = 2
    assert!(!errors.is_empty());
    let debug = format!("{node:#?}");
    assert!(debug.contains("ATTRIBUTE"));
}

// === Lossless round-trip through parser ===

#[test]
fn parser_lossless_roundtrip() {
    let inputs = [
        "x = 1\n",
        "name = \"hello\"\n",
        "resource \"aws\" \"x\" {\n  a = 1\n}\n",
        "x = 1 + 2 * 3\n",
        "x = a ? b : c\n",
        "x = [1, 2]\n",
        "x = {a = 1}\n",
    ];
    for input in inputs {
        let (green, _errors) = parse_file(input);
        let node = SyntaxNode::new_root(green);
        let reconstructed = node.text().to_string();
        assert_eq!(
            reconstructed, input,
            "Parser round-trip failed for: {input:?}"
        );
    }
}
