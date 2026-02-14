use crate::syntax_kind::SyntaxKind;

use super::Parser;

pub(crate) fn parse_expression(p: &mut Parser) {
    parse_conditional_expr(p);
}

fn parse_conditional_expr(p: &mut Parser) {
    let checkpoint = p.checkpoint();
    parse_binary_expr(p, 0);

    p.skip_trivia();
    if p.peek() == Some(SyntaxKind::QUESTION) {
        p.start_node_at(checkpoint, SyntaxKind::CONDITIONAL_EXPR);
        p.bump(); // ?
        p.skip_trivia();
        parse_expression(p);
        p.skip_trivia();
        p.expect(SyntaxKind::COLON);
        p.skip_trivia();
        parse_expression(p);
        p.finish_node();
    }
}

/// Returns (left_bp, right_bp) for binary operators, or None if not a binary op.
fn binary_binding_power(kind: SyntaxKind) -> Option<(u8, u8)> {
    match kind {
        SyntaxKind::PIPE_PIPE => Some((1, 2)),
        SyntaxKind::AMP_AMP => Some((3, 4)),
        SyntaxKind::EQ_EQ | SyntaxKind::BANG_EQ => Some((5, 6)),
        SyntaxKind::LT | SyntaxKind::LT_EQ | SyntaxKind::GT | SyntaxKind::GT_EQ => Some((7, 8)),
        SyntaxKind::PLUS | SyntaxKind::MINUS => Some((9, 10)),
        SyntaxKind::STAR | SyntaxKind::SLASH | SyntaxKind::PERCENT => Some((11, 12)),
        _ => None,
    }
}

fn parse_binary_expr(p: &mut Parser, min_bp: u8) {
    let checkpoint = p.checkpoint();
    parse_unary_expr(p);

    loop {
        p.skip_trivia();
        let op = match p.peek() {
            Some(kind) => kind,
            None => break,
        };

        let (left_bp, right_bp) = match binary_binding_power(op) {
            Some(bp) => bp,
            None => break,
        };

        if left_bp < min_bp {
            break;
        }

        p.start_node_at(checkpoint, SyntaxKind::BINARY_EXPR);
        p.bump(); // operator
        p.skip_trivia();
        parse_binary_expr(p, right_bp);
        p.finish_node();
    }
}

fn parse_unary_expr(p: &mut Parser) {
    match p.peek() {
        Some(SyntaxKind::MINUS) | Some(SyntaxKind::BANG) => {
            p.start_node(SyntaxKind::UNARY_EXPR);
            p.bump(); // operator
            p.skip_trivia();
            parse_unary_expr(p);
            p.finish_node();
        }
        _ => parse_postfix_expr(p),
    }
}

fn parse_postfix_expr(p: &mut Parser) {
    let checkpoint = p.checkpoint();
    parse_primary_expr(p);

    loop {
        p.skip_trivia();
        match p.peek() {
            Some(SyntaxKind::DOT) => {
                // Check for splat: .*
                if p.peek_non_trivia_nth(1) == Some(SyntaxKind::STAR) {
                    p.start_node_at(checkpoint, SyntaxKind::ATTR_SPLAT_EXPR);
                    p.bump(); // .
                    p.bump(); // *
                    parse_splat_body(p);
                    p.finish_node();
                } else {
                    p.start_node_at(checkpoint, SyntaxKind::ATTR_ACCESS_EXPR);
                    p.bump(); // .
                    p.skip_trivia();
                    if p.peek() == Some(SyntaxKind::IDENT) {
                        p.bump();
                    } else if p.peek() == Some(SyntaxKind::NUMBER) {
                        // Tuple access like tuple.0
                        p.bump();
                    } else {
                        p.expect(SyntaxKind::IDENT);
                    }
                    p.finish_node();
                }
            }
            Some(SyntaxKind::BRACKET_L) => {
                // Check for index splat: [*]
                if p.peek_non_trivia_nth(1) == Some(SyntaxKind::STAR) {
                    p.start_node_at(checkpoint, SyntaxKind::INDEX_SPLAT_EXPR);
                    p.bump(); // [
                    p.skip_trivia();
                    p.bump(); // *
                    p.skip_trivia();
                    p.expect(SyntaxKind::BRACKET_R);
                    parse_splat_body(p);
                    p.finish_node();
                } else {
                    p.start_node_at(checkpoint, SyntaxKind::INDEX_EXPR);
                    p.bump(); // [
                    p.skip_trivia();
                    parse_expression(p);
                    p.skip_trivia();
                    p.expect(SyntaxKind::BRACKET_R);
                    p.finish_node();
                }
            }
            _ => break,
        }
    }
}

fn parse_splat_body(p: &mut Parser) {
    let has_body = match p.peek_non_trivia() {
        Some(SyntaxKind::DOT) => true,
        Some(SyntaxKind::BRACKET_L) => {
            // [*] starts a new splat, not part of this splat body
            p.peek_non_trivia_nth(1) != Some(SyntaxKind::STAR)
        }
        _ => false,
    };
    if !has_body {
        return;
    }

    p.start_node(SyntaxKind::SPLAT_BODY);
    loop {
        p.skip_trivia();
        match p.peek() {
            Some(SyntaxKind::DOT) => {
                // .* starts a new splat - stop here
                if p.peek_non_trivia_nth(1) == Some(SyntaxKind::STAR) {
                    break;
                }
                p.start_node(SyntaxKind::ATTR_ACCESS_EXPR);
                p.bump(); // .
                p.skip_trivia();
                p.expect(SyntaxKind::IDENT);
                p.finish_node();
            }
            Some(SyntaxKind::BRACKET_L) => {
                // [*] starts a new splat - stop here
                if p.peek_non_trivia_nth(1) == Some(SyntaxKind::STAR) {
                    break;
                }
                p.start_node(SyntaxKind::INDEX_EXPR);
                p.bump(); // [
                p.skip_trivia();
                parse_expression(p);
                p.skip_trivia();
                p.expect(SyntaxKind::BRACKET_R);
                p.finish_node();
            }
            _ => break,
        }
    }
    p.finish_node();
}

fn parse_primary_expr(p: &mut Parser) {
    p.skip_trivia();
    match p.peek() {
        Some(SyntaxKind::NUMBER) => {
            p.start_node(SyntaxKind::LITERAL_EXPR);
            p.bump();
            p.finish_node();
        }
        Some(SyntaxKind::TRUE_KW | SyntaxKind::FALSE_KW | SyntaxKind::NULL_KW) => {
            p.start_node(SyntaxKind::LITERAL_EXPR);
            p.bump();
            p.finish_node();
        }
        Some(SyntaxKind::IDENT) => {
            // Check for function call: IDENT(
            if p.peek_non_trivia_nth(1) == Some(SyntaxKind::PAREN_L) {
                parse_function_call(p);
            } else {
                p.start_node(SyntaxKind::VARIABLE_EXPR);
                p.bump();
                p.finish_node();
            }
        }
        Some(SyntaxKind::QUOTE) => {
            super::template::parse_string_expr(p);
        }
        Some(SyntaxKind::HEREDOC_OPEN) => {
            super::template::parse_heredoc_expr(p);
        }
        Some(SyntaxKind::PAREN_L) => {
            parse_paren_expr(p);
        }
        Some(SyntaxKind::BRACKET_L) => {
            if is_for_expr(p) {
                parse_for_tuple_expr(p);
            } else {
                parse_tuple_expr(p);
            }
        }
        Some(SyntaxKind::BRACE_L) => {
            if is_for_expr(p) {
                parse_for_object_expr(p);
            } else {
                parse_object_expr(p);
            }
        }
        _ => {
            let offset = p.current_offset();
            let found = p
                .peek()
                .map(|k| format!("{:?}", k))
                .unwrap_or_else(|| "EOF".to_string());
            p.errors.push(crate::error::ParseError::new(
                format!("expected expression, found {}", found),
                offset,
            ));
            p.start_node(SyntaxKind::ERROR);
            if !p.at_end() {
                p.bump();
            }
            p.finish_node();
        }
    }
}

fn parse_function_call(p: &mut Parser) {
    p.start_node(SyntaxKind::FUNCTION_CALL);
    p.bump(); // IDENT
    p.skip_trivia();
    p.expect(SyntaxKind::PAREN_L);
    p.skip_trivia();

    p.start_node(SyntaxKind::ARG_LIST);
    if p.peek() != Some(SyntaxKind::PAREN_R) {
        parse_expression(p);
        loop {
            p.skip_trivia();
            if p.peek() == Some(SyntaxKind::COMMA) {
                p.bump();
                p.skip_trivia();
                if p.peek() == Some(SyntaxKind::PAREN_R) {
                    break; // trailing comma
                }
                if p.peek() == Some(SyntaxKind::ELLIPSIS) {
                    p.bump();
                    break;
                }
                parse_expression(p);
            } else if p.peek() == Some(SyntaxKind::ELLIPSIS) {
                p.bump();
                break;
            } else {
                break;
            }
        }
    }
    p.finish_node(); // ARG_LIST

    p.skip_trivia();
    p.expect(SyntaxKind::PAREN_R);
    p.finish_node(); // FUNCTION_CALL
}

fn parse_paren_expr(p: &mut Parser) {
    p.start_node(SyntaxKind::PAREN_EXPR);
    p.bump(); // (
    p.skip_trivia();
    parse_expression(p);
    p.skip_trivia();
    p.expect(SyntaxKind::PAREN_R);
    p.finish_node();
}

fn parse_tuple_expr(p: &mut Parser) {
    p.start_node(SyntaxKind::TUPLE_EXPR);
    p.bump(); // [
    p.skip_trivia();

    if p.peek() != Some(SyntaxKind::BRACKET_R) {
        parse_expression(p);
        loop {
            p.skip_trivia();
            if p.peek() == Some(SyntaxKind::COMMA) {
                p.bump();
                p.skip_trivia();
                if p.peek() == Some(SyntaxKind::BRACKET_R) {
                    break; // trailing comma
                }
                parse_expression(p);
            } else {
                break;
            }
        }
    }

    p.skip_trivia();
    p.expect(SyntaxKind::BRACKET_R);
    p.finish_node();
}

fn parse_object_expr(p: &mut Parser) {
    p.start_node(SyntaxKind::OBJECT_EXPR);
    p.bump(); // {
    p.skip_trivia();

    while p.peek() != Some(SyntaxKind::BRACE_R) && !p.at_end() {
        parse_object_elem(p);
        p.skip_trivia();
        if p.peek() == Some(SyntaxKind::COMMA) {
            p.bump();
        }
        p.skip_trivia();
    }

    p.expect(SyntaxKind::BRACE_R);
    p.finish_node();
}

fn parse_object_elem(p: &mut Parser) {
    p.start_node(SyntaxKind::OBJECT_ELEM);
    if p.peek() == Some(SyntaxKind::PAREN_L) {
        parse_paren_expr(p);
    } else {
        parse_expression(p);
    }
    p.skip_trivia();
    match p.peek() {
        Some(SyntaxKind::EQ) => p.bump(),
        Some(SyntaxKind::COLON) => p.bump(),
        Some(SyntaxKind::FAT_ARROW) => p.bump(),
        _ => {
            let offset = p.current_offset();
            p.errors.push(crate::error::ParseError::new(
                "expected '=', ':', or '=>' in object element",
                offset,
            ));
        }
    }
    p.skip_trivia();
    parse_expression(p);
    p.finish_node();
}

fn is_for_expr(p: &Parser) -> bool {
    p.peek_non_trivia_nth(1) == Some(SyntaxKind::FOR_KW)
}

fn parse_for_intro(p: &mut Parser) {
    p.start_node(SyntaxKind::FOR_INTRO);
    p.expect(SyntaxKind::FOR_KW);
    p.skip_trivia();
    p.expect(SyntaxKind::IDENT);
    p.skip_trivia();
    if p.peek() == Some(SyntaxKind::COMMA) {
        p.bump();
        p.skip_trivia();
        p.expect(SyntaxKind::IDENT);
        p.skip_trivia();
    }
    p.expect(SyntaxKind::IN_KW);
    p.skip_trivia();
    parse_expression(p);
    p.skip_trivia();
    p.expect(SyntaxKind::COLON);
    p.finish_node();
}

fn parse_for_cond(p: &mut Parser) {
    if p.peek_non_trivia() == Some(SyntaxKind::IF_KW) {
        p.skip_trivia();
        p.start_node(SyntaxKind::FOR_COND);
        p.bump(); // if
        p.skip_trivia();
        parse_expression(p);
        p.finish_node();
    }
}

fn parse_for_tuple_expr(p: &mut Parser) {
    p.start_node(SyntaxKind::FOR_TUPLE_EXPR);
    p.bump(); // [
    p.skip_trivia();
    parse_for_intro(p);
    p.skip_trivia();
    parse_expression(p);
    p.skip_trivia();
    parse_for_cond(p);
    p.skip_trivia();
    p.expect(SyntaxKind::BRACKET_R);
    p.finish_node();
}

fn parse_for_object_expr(p: &mut Parser) {
    p.start_node(SyntaxKind::FOR_OBJECT_EXPR);
    p.bump(); // {
    p.skip_trivia();
    parse_for_intro(p);
    p.skip_trivia();
    parse_expression(p); // key expr
    p.skip_trivia();
    p.expect(SyntaxKind::FAT_ARROW);
    p.skip_trivia();
    parse_expression(p); // value expr
    p.skip_trivia();
    if p.peek() == Some(SyntaxKind::ELLIPSIS) {
        p.bump();
    }
    p.skip_trivia();
    parse_for_cond(p);
    p.skip_trivia();
    p.expect(SyntaxKind::BRACE_R);
    p.finish_node();
}
