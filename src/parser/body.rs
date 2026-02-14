use crate::syntax_kind::SyntaxKind;

use super::Parser;

pub(crate) fn parse_source_file(p: &mut Parser) {
    p.start_node(SyntaxKind::SOURCE_FILE);
    parse_body(p);
    p.finish_node();
}

pub(crate) fn parse_body(p: &mut Parser) {
    p.start_node(SyntaxKind::BODY);
    loop {
        p.skip_trivia();
        if p.at_end() {
            break;
        }

        match p.peek() {
            Some(SyntaxKind::BRACE_R) => break, // end of block body
            Some(SyntaxKind::IDENT) => {
                // Lookahead to determine attribute vs block:
                // attribute: IDENT = expr
                // block:     IDENT [labels...] {
                match p.peek_non_trivia_nth(1) {
                    Some(SyntaxKind::EQ) => parse_attribute(p),
                    Some(
                        SyntaxKind::BRACE_L
                        | SyntaxKind::IDENT
                        | SyntaxKind::QUOTE
                        | SyntaxKind::STRING_LIT,
                    ) => parse_block(p),
                    _ => {
                        // Error recovery: unexpected token after IDENT
                        error_recover(p);
                    }
                }
            }
            _ => {
                error_recover(p);
            }
        }
    }
    p.finish_node();
}

fn parse_attribute(p: &mut Parser) {
    p.start_node(SyntaxKind::ATTRIBUTE);
    p.bump(); // IDENT
    p.skip_trivia();
    p.expect(SyntaxKind::EQ);
    p.skip_trivia();
    super::expr::parse_expression(p);
    // Consume trailing newline/trivia
    eat_trailing_newline(p);
    p.finish_node();
}

fn parse_block(p: &mut Parser) {
    p.start_node(SyntaxKind::BLOCK);
    p.bump(); // IDENT (block type)
    p.skip_trivia();

    // Parse labels (identifiers or quoted strings)
    loop {
        match p.peek() {
            Some(SyntaxKind::IDENT) => {
                p.start_node(SyntaxKind::BLOCK_LABEL);
                p.bump();
                p.finish_node();
                p.skip_trivia();
            }
            Some(SyntaxKind::QUOTE) => {
                p.start_node(SyntaxKind::BLOCK_LABEL);
                super::template::parse_string_expr(p);
                p.finish_node();
                p.skip_trivia();
            }
            _ => break,
        }
    }

    p.expect(SyntaxKind::BRACE_L);
    // Consume newline after opening brace
    eat_trailing_newline(p);

    parse_body(p);

    p.skip_trivia();
    p.expect(SyntaxKind::BRACE_R);
    eat_trailing_newline(p);
    p.finish_node();
}

fn eat_trailing_newline(p: &mut Parser) {
    // Eat whitespace and at most one newline
    while let Some(kind) = p.peek() {
        match kind {
            SyntaxKind::WHITESPACE => p.bump(),
            SyntaxKind::NEWLINE => {
                p.bump();
                break;
            }
            SyntaxKind::LINE_COMMENT => {
                p.bump();
                // The newline after a line comment
                if p.peek() == Some(SyntaxKind::NEWLINE) {
                    p.bump();
                }
                break;
            }
            _ => break,
        }
    }
}

fn error_recover(p: &mut Parser) {
    let offset = p.current_offset();
    p.errors.push(crate::error::ParseError::new(
        format!(
            "unexpected token {:?}",
            p.peek().unwrap_or(SyntaxKind::ERROR_TOKEN)
        ),
        offset,
    ));
    p.start_node(SyntaxKind::ERROR);
    // Skip tokens until we find a recovery point
    while let Some(kind) = p.peek() {
        match kind {
            SyntaxKind::NEWLINE => {
                p.bump();
                break;
            }
            SyntaxKind::BRACE_R => break, // don't consume the closing brace
            _ => p.bump(),
        }
    }
    p.finish_node();
}
