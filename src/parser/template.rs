use crate::syntax_kind::SyntaxKind;

use super::Parser;

pub(crate) fn parse_string_expr(p: &mut Parser) {
    p.start_node(SyntaxKind::STRING_EXPR);
    p.expect(SyntaxKind::QUOTE); // opening quote

    loop {
        match p.peek() {
            Some(SyntaxKind::QUOTE) => {
                p.bump(); // closing quote
                break;
            }
            Some(SyntaxKind::STRING_FRAGMENT) => {
                p.bump();
            }
            Some(SyntaxKind::ESCAPE_SEQUENCE) => {
                p.bump();
            }
            Some(SyntaxKind::DOLLAR_OPEN) => {
                parse_template_interpolation(p);
            }
            Some(SyntaxKind::PERCENT_OPEN) => {
                parse_template_directive(p);
            }
            None => {
                let offset = p.current_offset();
                p.errors.push(crate::error::ParseError::new(
                    "unterminated string",
                    offset,
                ));
                break;
            }
            _ => {
                // Unexpected token inside string - might happen with errors
                p.bump();
            }
        }
    }

    p.finish_node();
}

pub(crate) fn parse_heredoc_expr(p: &mut Parser) {
    p.start_node(SyntaxKind::HEREDOC_EXPR);
    p.bump(); // HEREDOC_OPEN

    loop {
        match p.peek() {
            Some(SyntaxKind::HEREDOC_ANCHOR) => {
                p.bump(); // closing anchor
                break;
            }
            Some(SyntaxKind::HEREDOC_CONTENT) => {
                p.bump();
            }
            Some(SyntaxKind::DOLLAR_OPEN) => {
                parse_template_interpolation(p);
            }
            Some(SyntaxKind::PERCENT_OPEN) => {
                parse_template_directive(p);
            }
            None => {
                let offset = p.current_offset();
                p.errors.push(crate::error::ParseError::new(
                    "unterminated heredoc",
                    offset,
                ));
                break;
            }
            _ => {
                p.bump();
            }
        }
    }

    p.finish_node();
}

fn parse_template_interpolation(p: &mut Parser) {
    p.start_node(SyntaxKind::TEMPLATE_INTERPOLATION);
    p.bump(); // DOLLAR_OPEN (${)

    // Optional tilde for strip marker
    if p.peek() == Some(SyntaxKind::TILDE) {
        p.bump();
    }

    p.skip_trivia();
    super::expr::parse_expression(p);
    p.skip_trivia();

    // Optional tilde before closing
    if p.peek() == Some(SyntaxKind::TILDE) {
        p.bump();
    }

    p.expect(SyntaxKind::TEMPLATE_CLOSE); // }
    p.finish_node();
}

fn parse_template_directive(p: &mut Parser) {
    p.start_node(SyntaxKind::TEMPLATE_DIRECTIVE);
    p.bump(); // PERCENT_OPEN (%{)

    // Optional tilde for strip marker
    if p.peek() == Some(SyntaxKind::TILDE) {
        p.bump();
    }

    p.skip_trivia();

    // Directive keyword: if, else, endif, for, endfor
    match p.peek() {
        Some(SyntaxKind::IF_KW) => {
            p.bump();
            p.skip_trivia();
            super::expr::parse_expression(p);
        }
        Some(SyntaxKind::ELSE_KW) => {
            p.bump();
        }
        Some(SyntaxKind::ENDIF_KW) => {
            p.bump();
        }
        Some(SyntaxKind::FOR_KW) => {
            p.bump();
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
            super::expr::parse_expression(p);
        }
        Some(SyntaxKind::ENDFOR_KW) => {
            p.bump();
        }
        _ => {
            let offset = p.current_offset();
            p.errors.push(crate::error::ParseError::new(
                "expected directive keyword (if, else, endif, for, endfor)",
                offset,
            ));
        }
    }

    p.skip_trivia();

    // Optional tilde before closing
    if p.peek() == Some(SyntaxKind::TILDE) {
        p.bump();
    }

    p.expect(SyntaxKind::TEMPLATE_CLOSE); // }
    p.finish_node();
}
