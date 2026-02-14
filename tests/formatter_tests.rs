use expect_test::{expect, Expect};
use stanu::formatter::{format, FormatResult};

fn check_fmt(input: &str, expected: Expect) {
    match format(input) {
        FormatResult::Changed(output) | FormatResult::Unchanged(output) => {
            expected.assert_eq(&output);
        }
        FormatResult::Skipped => {
            panic!("format() returned Skipped for input:\n{input}");
        }
    }
}

fn check_unchanged(input: &str) {
    match format(input) {
        FormatResult::Unchanged(_) => {}
        FormatResult::Changed(output) => {
            panic!(
                "Expected unchanged, but got changed.\nInput:\n{input}\nOutput:\n{output}"
            );
        }
        FormatResult::Skipped => {
            panic!("format() returned Skipped for input:\n{input}");
        }
    }
}

fn check_idempotent(input: &str) {
    let first = match format(input) {
        FormatResult::Changed(output) | FormatResult::Unchanged(output) => output,
        FormatResult::Skipped => panic!("format() returned Skipped"),
    };
    match format(&first) {
        FormatResult::Unchanged(_) => {}
        FormatResult::Changed(second) => {
            panic!(
                "Not idempotent!\nFirst pass:\n{first}\nSecond pass:\n{second}"
            );
        }
        FormatResult::Skipped => panic!("Second format() returned Skipped"),
    }
}

// === Basic attribute formatting ===

#[test]
fn simple_attribute() {
    check_fmt(
        "x = 1\n",
        expect![[r#"
            x = 1
        "#]],
    );
}

#[test]
fn attribute_no_spaces_around_eq() {
    check_fmt(
        "x=1\n",
        expect![[r#"
            x = 1
        "#]],
    );
}

#[test]
fn attribute_extra_spaces() {
    check_fmt(
        "x   =   1\n",
        expect![[r#"
            x = 1
        "#]],
    );
}

// === Attribute alignment ===

#[test]
fn attribute_alignment() {
    check_fmt(
        "ami = \"ami-123\"\ninstance_type = \"t2.micro\"\n",
        expect![[r#"
            ami           = "ami-123"
            instance_type = "t2.micro"
        "#]],
    );
}

#[test]
fn alignment_broken_by_blank_line() {
    check_fmt(
        "short = 1\n\nlong_name = 2\n",
        expect![[r#"
            short = 1

            long_name = 2
        "#]],
    );
}

// === Block formatting ===

#[test]
fn simple_block() {
    check_fmt(
        "resource \"aws\" \"x\" {\n  a = 1\n}\n",
        expect![[r#"
            resource "aws" "x" {
              a = 1
            }
        "#]],
    );
}

#[test]
fn block_wrong_indent() {
    check_fmt(
        "resource \"aws\" \"x\" {\na = 1\n}\n",
        expect![[r#"
            resource "aws" "x" {
              a = 1
            }
        "#]],
    );
}

#[test]
fn nested_blocks() {
    check_fmt(
        "outer {\ninner {\nx = 1\n}\n}\n",
        expect![[r#"
            outer {
              inner {
                x = 1
              }
            }
        "#]],
    );
}

#[test]
fn blank_line_before_block_in_body() {
    check_fmt(
        "a = 1\nresource {\n  b = 2\n}\n",
        expect![[r#"
            a = 1

            resource {
              b = 2
            }
        "#]],
    );
}

// === Expression formatting ===

#[test]
fn binary_expr_spaces() {
    check_fmt(
        "x = 1+2\n",
        expect![[r#"
            x = 1 + 2
        "#]],
    );
}

#[test]
fn unary_expr() {
    check_fmt(
        "x = -5\n",
        expect![[r#"
            x = -5
        "#]],
    );
}

#[test]
fn conditional_expr() {
    check_fmt(
        "x = a?b:c\n",
        expect![[r#"
            x = a ? b : c
        "#]],
    );
}

#[test]
fn function_call() {
    check_fmt(
        "x = length( list )\n",
        expect![[r#"
            x = length(list)
        "#]],
    );
}

#[test]
fn function_call_multi_args() {
    check_fmt(
        "x = join(\",\",list)\n",
        expect![[r#"
            x = join(",", list)
        "#]],
    );
}

#[test]
fn tuple_inline() {
    check_fmt(
        "x = [1,2,3]\n",
        expect![[r#"
            x = [1, 2, 3]
        "#]],
    );
}

#[test]
fn object_inline() {
    check_fmt(
        "x = {a = 1, b = 2}\n",
        expect![[r#"
            x = {a = 1, b = 2}
        "#]],
    );
}

#[test]
fn attr_access() {
    check_fmt(
        "x = a.b.c\n",
        expect![[r#"
            x = a.b.c
        "#]],
    );
}

#[test]
fn index_expr() {
    check_fmt(
        "x = a[0]\n",
        expect![[r#"
            x = a[0]
        "#]],
    );
}

#[test]
fn splat_expr() {
    check_fmt(
        "x = items.*.name\n",
        expect![[r#"
            x = items.*.name
        "#]],
    );
}

#[test]
fn for_tuple_expr() {
    check_fmt(
        "x = [for s in list : upper(s) if s != \"\"]\n",
        expect![[r#"
            x = [for s in list : upper(s) if s != ""]
        "#]],
    );
}

#[test]
fn for_object_expr() {
    check_fmt(
        "x = {for k, v in map : k => upper(v)}\n",
        expect![[r#"
            x = {for k, v in map : k => upper(v)}
        "#]],
    );
}

#[test]
fn paren_expr() {
    check_fmt(
        "x = (1 + 2) * 3\n",
        expect![[r#"
            x = (1 + 2) * 3
        "#]],
    );
}

// === String and heredoc preservation ===

#[test]
fn string_preserved_verbatim() {
    check_fmt(
        "x = \"hello ${var.name} world\"\n",
        expect![[r#"
            x = "hello ${var.name} world"
        "#]],
    );
}

#[test]
fn heredoc_preserved_verbatim() {
    check_fmt(
        "x = <<EOF\nhello\n  world\nEOF\n",
        expect![[r#"
            x = <<EOF
            hello
              world
            EOF
        "#]],
    );
}

// === Comment handling ===

#[test]
fn trailing_comment_on_attribute() {
    check_fmt(
        "x = 1 # this is x\n",
        expect![[r#"
            x = 1 # this is x
        "#]],
    );
}

// === Multi-line value doesn't align ===

#[test]
fn multiline_value_breaks_alignment() {
    // Multiline values break alignment groups (no padding on short)
    check_fmt(
        "short = \"a\"\nlong_name = {\n  x = 1\n}\n",
        expect![[r#"
            short = "a"
            long_name = {
              x = 1
            }
        "#]],
    );
}

// === Parse errors cause skip ===

#[test]
fn parse_errors_skip() {
    let result = format("!!!\n");
    assert!(matches!(result, FormatResult::Skipped));
}

// === Idempotency ===

#[test]
fn idempotent_simple() {
    check_idempotent("variable \"name\" {\n  type    = string\n  default = \"hello\"\n}\n");
}

#[test]
fn idempotent_complex() {
    check_idempotent(
        r#"locals {
  a = 1 + 2 * 3
  b = var.enabled ? "yes" : "no"
  c = length(var.list)
  d = [for s in var.list : upper(s) if s != ""]
  e = {for k, v in var.map : k => upper(v)}
  f = var.items[*].name
  g = -5
  h = !var.flag
  i = (1 + 2) * 3
  j = [1, 2, 3]
  k = {a = 1, b = 2}
  l = "hello ${var.name} world"
}
"#,
    );
}

// === Fixture idempotency ===

#[test]
fn fixture_simple_tf_idempotent() {
    let source = include_str!("fixtures/simple.tf");
    check_unchanged(source);
}

#[test]
fn fixture_expressions_tf_idempotent() {
    let source = include_str!("fixtures/expressions.tf");
    check_unchanged(source);
}

#[test]
fn fixture_heredoc_tf_idempotent() {
    let source = include_str!("fixtures/heredoc.tf");
    check_unchanged(source);
}

// === Full formatting test ===

#[test]
fn full_formatting() {
    check_fmt(
        r#"variable    "name"     {
type=string
default="hello"
}


resource "aws_instance"   "web" {
    ami = "ami-12345"
    instance_type="t2.micro"

    tags={
    Name="web-server"
    }
}
"#,
        expect![[r#"
            variable "name" {
              type    = string
              default = "hello"
            }

            resource "aws_instance" "web" {
              ami           = "ami-12345"
              instance_type = "t2.micro"

              tags = {
                Name = "web-server"
              }
            }
        "#]],
    );
}
