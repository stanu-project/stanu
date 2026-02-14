use std::fs;
use std::io;
use std::path::Path;

use rowan::NodeOrToken;

use crate::parse_file;
use crate::syntax_kind::{SyntaxElement, SyntaxKind, SyntaxNode};

#[derive(Debug, PartialEq, Eq)]
pub enum FormatResult {
    Unchanged(String),
    Changed(String),
    Skipped,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FormatStatus {
    Unchanged,
    Changed,
    Skipped,
}

pub fn format(source: &str) -> FormatResult {
    let (green, errors) = parse_file(source);
    if !errors.is_empty() {
        return FormatResult::Skipped;
    }
    let root = SyntaxNode::new_root(green);
    let mut f = Formatter::new();
    f.format_node(&root);
    let mut output = f.buf;
    // Ensure file ends with single newline
    let trimmed = output.trim_end_matches('\n');
    output.truncate(trimmed.len());
    output.push('\n');

    if output == source {
        FormatResult::Unchanged(output)
    } else {
        FormatResult::Changed(output)
    }
}

pub fn format_file(path: &Path, check_only: bool) -> io::Result<FormatStatus> {
    let source = fs::read_to_string(path)?;
    match format(&source) {
        FormatResult::Unchanged(_) => Ok(FormatStatus::Unchanged),
        FormatResult::Changed(formatted) => {
            if !check_only {
                fs::write(path, &formatted)?;
            }
            Ok(FormatStatus::Changed)
        }
        FormatResult::Skipped => Ok(FormatStatus::Skipped),
    }
}

struct Formatter {
    buf: String,
    indent: usize,
}

const INDENT: &str = "  ";

impl Formatter {
    fn new() -> Self {
        Self {
            buf: String::new(),
            indent: 0,
        }
    }

    fn write(&mut self, s: &str) {
        self.buf.push_str(s);
    }

    fn newline(&mut self) {
        self.buf.push('\n');
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.buf.push_str(INDENT);
        }
    }

    fn format_node(&mut self, node: &SyntaxNode) {
        match node.kind() {
            SyntaxKind::SOURCE_FILE => self.format_source_file(node),
            _ => self.format_expr(node),
        }
    }

    fn format_source_file(&mut self, node: &SyntaxNode) {
        for child in node.children() {
            match child.kind() {
                SyntaxKind::BODY => self.format_body(&child),
                _ => {}
            }
        }
    }

    // ── Body formatting with alignment groups ─────────────────────

    fn format_body(&mut self, node: &SyntaxNode) {
        let items = self.classify_body_items(node);
        let groups = self.compute_alignment_groups(&items);

        let mut prev_kind = PrevItemKind::None;

        for (i, item) in items.iter().enumerate() {
            match item {
                BodyItem::Attribute { node: attr, .. } => {
                    if prev_kind == PrevItemKind::Block {
                        self.newline();
                    }
                    let max_key = groups.iter().find_map(|g| {
                        if i >= g.start && i < g.end {
                            Some(g.max_key_len)
                        } else {
                            None
                        }
                    });
                    self.format_attribute(attr, max_key);
                    prev_kind = PrevItemKind::Attribute;
                }
                BodyItem::Block { node: blk } => {
                    if prev_kind != PrevItemKind::None && prev_kind != PrevItemKind::Comment {
                        self.newline();
                    }
                    self.format_block(blk);
                    prev_kind = PrevItemKind::Block;
                }
                BodyItem::BlankLine => {
                    if prev_kind != PrevItemKind::None && !self.buf.ends_with("\n\n") {
                        self.newline();
                    }
                    prev_kind = PrevItemKind::BlankLine;
                }
                BodyItem::Comment { text } => {
                    if prev_kind == PrevItemKind::Block
                        || prev_kind == PrevItemKind::Attribute
                    {
                        self.newline();
                    }
                    self.write_indent();
                    self.write(text.trim_end());
                    self.newline();
                    prev_kind = PrevItemKind::Comment;
                }
            }
        }
    }

    fn classify_body_items(&self, node: &SyntaxNode) -> Vec<BodyItem> {
        let mut items = Vec::new();

        for elem in node.children_with_tokens() {
            match elem {
                NodeOrToken::Node(ref child) => match child.kind() {
                    SyntaxKind::ATTRIBUTE => {
                        let key_len = self.attribute_key_len(child);
                        let has_blank_after = self.node_has_trailing_blank_line(child);
                        let has_multiline_value = self.attribute_has_multiline_value(child);
                        items.push(BodyItem::Attribute {
                            node: child.clone(),
                            key_len,
                            multiline_value: has_multiline_value,
                        });
                        if has_blank_after {
                            items.push(BodyItem::BlankLine);
                        }
                    }
                    SyntaxKind::BLOCK => {
                        items.push(BodyItem::Block {
                            node: child.clone(),
                        });
                    }
                    _ => {}
                },
                NodeOrToken::Token(ref tok) => match tok.kind() {
                    SyntaxKind::LINE_COMMENT | SyntaxKind::BLOCK_COMMENT => {
                        items.push(BodyItem::Comment {
                            text: tok.text().to_string(),
                        });
                    }
                    _ => {}
                },
            }
        }
        items
    }

    fn node_has_trailing_blank_line(&self, node: &SyntaxNode) -> bool {
        let mut newline_count = 0;
        let elems: Vec<SyntaxElement> = node.children_with_tokens().collect();
        for elem in elems.iter().rev() {
            if let NodeOrToken::Token(ref tok) = elem {
                match tok.kind() {
                    SyntaxKind::NEWLINE => newline_count += 1,
                    SyntaxKind::WHITESPACE => {}
                    _ => break,
                }
            } else {
                break;
            }
        }
        newline_count >= 2
    }

    fn attribute_key_len(&self, attr: &SyntaxNode) -> usize {
        for elem in attr.children_with_tokens() {
            match elem {
                NodeOrToken::Token(ref tok) => {
                    if is_ident_like(tok.kind()) {
                        return tok.text().len();
                    }
                }
                _ => {}
            }
        }
        0
    }

    fn attribute_has_multiline_value(&self, attr: &SyntaxNode) -> bool {
        if let Some(expr) = self.find_attribute_expr(attr) {
            node_contains_newline_recursive(&expr)
        } else {
            false
        }
    }

    fn compute_alignment_groups(&self, items: &[BodyItem]) -> Vec<AlignGroup> {
        let mut groups = Vec::new();
        let mut group_start: Option<usize> = None;
        let mut max_key: usize = 0;

        for (i, item) in items.iter().enumerate() {
            match item {
                BodyItem::Attribute {
                    key_len,
                    multiline_value,
                    ..
                } => {
                    if *multiline_value {
                        // Multi-line attributes break alignment groups
                        if let Some(start) = group_start {
                            groups.push(AlignGroup {
                                start,
                                end: i,
                                max_key_len: max_key,
                            });
                            group_start = None;
                        }
                        // This attribute is its own "group" of size 1 (no alignment)
                    } else {
                        if group_start.is_none() {
                            group_start = Some(i);
                            max_key = 0;
                        }
                        max_key = max_key.max(*key_len);
                    }
                }
                BodyItem::Comment { .. } => {
                    // Comments don't break alignment groups
                }
                _ => {
                    // Blank lines and blocks break groups
                    if let Some(start) = group_start {
                        groups.push(AlignGroup {
                            start,
                            end: i,
                            max_key_len: max_key,
                        });
                        group_start = None;
                    }
                }
            }
        }
        if let Some(start) = group_start {
            groups.push(AlignGroup {
                start,
                end: items.len(),
                max_key_len: max_key,
            });
        }
        groups
    }

    // ── Attribute formatting ──────────────────────────────────────

    fn format_attribute(&mut self, node: &SyntaxNode, align_to: Option<usize>) {
        self.write_indent();

        let mut key_text = String::new();
        let mut trailing_comment: Option<String> = None;
        let mut saw_eq = false;

        // Extract key and trailing comment
        for elem in node.children_with_tokens() {
            if let NodeOrToken::Token(ref tok) = elem {
                match tok.kind() {
                    k if is_ident_like(k) && !saw_eq => {
                        key_text = tok.text().to_string();
                    }
                    SyntaxKind::EQ => {
                        saw_eq = true;
                    }
                    SyntaxKind::LINE_COMMENT | SyntaxKind::BLOCK_COMMENT => {
                        trailing_comment = Some(tok.text().to_string());
                    }
                    _ => {}
                }
            }
        }

        // Write key with alignment padding
        self.write(&key_text);
        if let Some(align) = align_to {
            let padding = align.saturating_sub(key_text.len());
            for _ in 0..padding {
                self.buf.push(' ');
            }
        }
        self.write(" = ");

        // Write value expression
        if let Some(expr) = self.find_attribute_expr(node) {
            self.format_expr(&expr);
        }

        if let Some(comment) = trailing_comment {
            self.write(" ");
            self.write(comment.trim_end());
        }
        self.newline();
    }

    fn find_attribute_expr(&self, attr: &SyntaxNode) -> Option<SyntaxNode> {
        let mut past_eq = false;
        for elem in attr.children_with_tokens() {
            match elem {
                NodeOrToken::Token(ref tok) if tok.kind() == SyntaxKind::EQ => {
                    past_eq = true;
                }
                NodeOrToken::Node(ref child) if past_eq => {
                    return Some(child.clone());
                }
                _ => {}
            }
        }
        None
    }

    // ── Block formatting ──────────────────────────────────────────

    fn format_block(&mut self, node: &SyntaxNode) {
        self.write_indent();

        // Write block type
        let mut labels = Vec::new();
        let mut block_type = String::new();
        let mut body_node: Option<SyntaxNode> = None;
        let mut trailing_comment: Option<String> = None;

        for elem in node.children_with_tokens() {
            match elem {
                NodeOrToken::Token(ref tok) => match tok.kind() {
                    k if is_ident_like(k) && block_type.is_empty() => {
                        block_type = tok.text().to_string();
                    }
                    SyntaxKind::LINE_COMMENT | SyntaxKind::BLOCK_COMMENT => {
                        trailing_comment = Some(tok.text().to_string());
                    }
                    _ => {}
                },
                NodeOrToken::Node(ref child) => match child.kind() {
                    SyntaxKind::BLOCK_LABEL => {
                        labels.push(child.clone());
                    }
                    SyntaxKind::BODY => {
                        body_node = Some(child.clone());
                    }
                    _ => {}
                },
            }
        }

        self.write(&block_type);
        for label in &labels {
            self.write(" ");
            self.format_block_label(label);
        }
        self.write(" {");
        self.newline();

        if let Some(body) = body_node {
            self.indent += 1;
            self.format_body(&body);
            self.indent -= 1;
        }

        self.write_indent();
        self.write("}");
        if let Some(comment) = trailing_comment {
            self.write(" ");
            self.write(comment.trim_end());
        }
        self.newline();
    }

    fn format_block_label(&mut self, node: &SyntaxNode) {
        // A label is either an ident or a string expr
        for child in node.children() {
            match child.kind() {
                SyntaxKind::STRING_EXPR => self.format_string_expr(&child),
                _ => {
                    // Ident-based label
                    self.write(&child.text().to_string());
                }
            }
        }
        // Also check for bare ident tokens
        for elem in node.children_with_tokens() {
            if let NodeOrToken::Token(ref tok) = elem {
                if is_ident_like(tok.kind()) {
                    self.write(tok.text());
                }
            }
        }
    }

    // ── Expression formatting ─────────────────────────────────────

    fn format_expr(&mut self, node: &SyntaxNode) {
        match node.kind() {
            SyntaxKind::LITERAL_EXPR => self.format_literal(node),
            SyntaxKind::STRING_EXPR => self.format_string_expr(node),
            SyntaxKind::HEREDOC_EXPR => self.format_heredoc(node),
            SyntaxKind::VARIABLE_EXPR => self.format_variable(node),
            SyntaxKind::BINARY_EXPR => self.format_binary_expr(node),
            SyntaxKind::UNARY_EXPR => self.format_unary_expr(node),
            SyntaxKind::CONDITIONAL_EXPR => self.format_conditional_expr(node),
            SyntaxKind::FUNCTION_CALL => self.format_function_call(node),
            SyntaxKind::PAREN_EXPR => self.format_paren_expr(node),
            SyntaxKind::TUPLE_EXPR => self.format_tuple_expr(node),
            SyntaxKind::OBJECT_EXPR => self.format_object_expr(node),
            SyntaxKind::ATTR_ACCESS_EXPR => self.format_attr_access(node),
            SyntaxKind::INDEX_EXPR => self.format_index_expr(node),
            SyntaxKind::ATTR_SPLAT_EXPR => self.format_attr_splat(node),
            SyntaxKind::INDEX_SPLAT_EXPR => self.format_index_splat(node),
            SyntaxKind::FOR_TUPLE_EXPR => self.format_for_tuple(node),
            SyntaxKind::FOR_OBJECT_EXPR => self.format_for_object(node),
            _ => {
                // Fallback: emit verbatim
                self.write(&node.text().to_string());
            }
        }
    }

    fn format_literal(&mut self, node: &SyntaxNode) {
        for elem in node.children_with_tokens() {
            if let NodeOrToken::Token(ref tok) = elem {
                if !is_trivia(tok.kind()) {
                    self.write(tok.text());
                }
            }
        }
    }

    fn format_variable(&mut self, node: &SyntaxNode) {
        for elem in node.children_with_tokens() {
            if let NodeOrToken::Token(ref tok) = elem {
                if !is_trivia(tok.kind()) {
                    self.write(tok.text());
                }
            }
        }
    }

    fn format_string_expr(&mut self, node: &SyntaxNode) {
        // Strings are preserved verbatim
        self.write(&node.text().to_string());
    }

    fn format_heredoc(&mut self, node: &SyntaxNode) {
        // Heredocs are preserved verbatim
        self.write(&node.text().to_string());
    }

    fn format_binary_expr(&mut self, node: &SyntaxNode) {
        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Node(ref child) => {
                    self.format_expr(child);
                }
                NodeOrToken::Token(ref tok) => {
                    if is_binary_op(tok.kind()) {
                        self.write(" ");
                        self.write(tok.text());
                        self.write(" ");
                    }
                }
            }
        }
    }

    fn format_unary_expr(&mut self, node: &SyntaxNode) {
        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Token(ref tok) => {
                    self.write(tok.text()); // operator, no space after
                }
                NodeOrToken::Node(ref child) => {
                    self.format_expr(child);
                }
            }
        }
    }

    fn format_conditional_expr(&mut self, node: &SyntaxNode) {
        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Token(ref tok) => match tok.kind() {
                    SyntaxKind::QUESTION => self.write(" ? "),
                    SyntaxKind::COLON => self.write(" : "),
                    _ => {}
                },
                NodeOrToken::Node(ref child) => {
                    self.format_expr(child);
                }
            }
        }
    }

    fn format_function_call(&mut self, node: &SyntaxNode) {
        let mut wrote_name = false;
        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Token(ref tok) => match tok.kind() {
                    SyntaxKind::IDENT if !wrote_name => {
                        self.write(tok.text());
                        wrote_name = true;
                    }
                    SyntaxKind::PAREN_L => self.write("("),
                    SyntaxKind::PAREN_R => self.write(")"),
                    _ => {}
                },
                NodeOrToken::Node(ref child) => match child.kind() {
                    SyntaxKind::ARG_LIST => self.format_arg_list(child),
                    _ => self.format_expr(child),
                },
            }
        }
    }

    fn format_arg_list(&mut self, node: &SyntaxNode) {
        let is_multiline = node_contains_newline(node);
        if is_multiline {
            self.format_arg_list_multiline(node);
        } else {
            self.format_arg_list_inline(node);
        }
    }

    fn format_arg_list_inline(&mut self, node: &SyntaxNode) {
        let mut first = true;
        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Token(ref tok) => match tok.kind() {
                    SyntaxKind::COMMA => {
                        self.write(", ");
                    }
                    SyntaxKind::ELLIPSIS => {
                        self.write("...");
                    }
                    _ => {}
                },
                NodeOrToken::Node(ref child) => {
                    if !first {
                        // comma already written
                    }
                    self.format_expr(child);
                    first = false;
                }
            }
        }
    }

    fn format_arg_list_multiline(&mut self, node: &SyntaxNode) {
        self.newline();
        self.indent += 1;
        let mut first = true;
        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Token(ref tok) => match tok.kind() {
                    SyntaxKind::COMMA => {}
                    SyntaxKind::ELLIPSIS => {
                        self.write("...");
                    }
                    _ => {}
                },
                NodeOrToken::Node(ref child) => {
                    if !first {
                        self.write(",");
                        self.newline();
                    }
                    self.write_indent();
                    self.format_expr(child);
                    first = false;
                }
            }
        }
        self.write(",");
        self.newline();
        self.indent -= 1;
        self.write_indent();
    }

    fn format_paren_expr(&mut self, node: &SyntaxNode) {
        self.write("(");
        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Token(ref tok) => match tok.kind() {
                    SyntaxKind::PAREN_L | SyntaxKind::PAREN_R => {}
                    _ => self.write(tok.text()),
                },
                NodeOrToken::Node(ref child) => {
                    self.format_expr(child);
                }
            }
        }
        self.write(")");
    }

    fn format_tuple_expr(&mut self, node: &SyntaxNode) {
        let is_multiline = node_contains_newline(node);
        if is_multiline {
            self.format_tuple_multiline(node);
        } else {
            self.format_tuple_inline(node);
        }
    }

    fn format_tuple_inline(&mut self, node: &SyntaxNode) {
        self.write("[");
        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Token(ref tok) => match tok.kind() {
                    SyntaxKind::BRACKET_L | SyntaxKind::BRACKET_R => {}
                    SyntaxKind::COMMA => self.write(", "),
                    _ => self.write(tok.text()),
                },
                NodeOrToken::Node(ref child) => {
                    self.format_expr(child);
                }
            }
        }
        self.write("]");
    }

    fn format_tuple_multiline(&mut self, node: &SyntaxNode) {
        self.write("[");
        self.newline();
        self.indent += 1;
        let mut first = true;
        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Token(ref tok) => match tok.kind() {
                    SyntaxKind::BRACKET_L | SyntaxKind::BRACKET_R | SyntaxKind::COMMA => {}
                    _ => self.write(tok.text()),
                },
                NodeOrToken::Node(ref child) => {
                    if !first {
                        self.write(",");
                        self.newline();
                    }
                    self.write_indent();
                    self.format_expr(child);
                    first = false;
                }
            }
        }
        self.write(",");
        self.newline();
        self.indent -= 1;
        self.write_indent();
        self.write("]");
    }

    fn format_object_expr(&mut self, node: &SyntaxNode) {
        let is_multiline = node_contains_newline(node);
        if is_multiline {
            self.format_object_multiline(node);
        } else {
            self.format_object_inline(node);
        }
    }

    fn format_object_inline(&mut self, node: &SyntaxNode) {
        let elems: Vec<SyntaxNode> = node
            .children()
            .filter(|c| c.kind() == SyntaxKind::OBJECT_ELEM)
            .collect();

        if elems.is_empty() {
            self.write("{}");
            return;
        }

        self.write("{");
        let mut first = true;
        for elem in &elems {
            if !first {
                self.write(", ");
            }
            self.format_object_elem_inline(elem);
            first = false;
        }
        self.write("}");
    }

    fn format_object_multiline(&mut self, node: &SyntaxNode) {
        self.write("{");
        self.newline();
        self.indent += 1;

        let elems: Vec<SyntaxNode> = node
            .children()
            .filter(|c| c.kind() == SyntaxKind::OBJECT_ELEM)
            .collect();

        // Compute alignment for object elements
        let max_key_len = elems.iter().map(|e| self.object_elem_key_len(e)).max().unwrap_or(0);

        for elem in &elems {
            self.write_indent();
            self.format_object_elem_aligned(elem, max_key_len);
            self.newline();
        }

        self.indent -= 1;
        self.write_indent();
        self.write("}");
    }

    fn object_elem_key_len(&self, node: &SyntaxNode) -> usize {
        // Key is the first expression or token before = or :
        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Token(ref tok) => {
                    if tok.kind() == SyntaxKind::EQ
                        || tok.kind() == SyntaxKind::COLON
                        || tok.kind() == SyntaxKind::FAT_ARROW
                    {
                        break;
                    }
                }
                NodeOrToken::Node(ref child) => {
                    return child.text().to_string().trim().len();
                }
            }
        }
        0
    }

    fn format_object_elem_inline(&mut self, node: &SyntaxNode) {
        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Token(ref tok) => match tok.kind() {
                    SyntaxKind::EQ | SyntaxKind::COLON => self.write(" = "),
                    SyntaxKind::FAT_ARROW => self.write(" => "),
                    _ => {}
                },
                NodeOrToken::Node(ref child) => {
                    self.format_expr(child);
                }
            }
        }
    }

    fn format_object_elem_aligned(&mut self, node: &SyntaxNode, max_key_len: usize) {
        let key_len = self.object_elem_key_len(node);

        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Token(ref tok) => match tok.kind() {
                    SyntaxKind::EQ | SyntaxKind::COLON => {
                        let padding = max_key_len.saturating_sub(key_len);
                        for _ in 0..padding {
                            self.buf.push(' ');
                        }
                        self.write(" = ");
                    }
                    SyntaxKind::FAT_ARROW => {
                        let padding = max_key_len.saturating_sub(key_len);
                        for _ in 0..padding {
                            self.buf.push(' ');
                        }
                        self.write(" => ");
                    }
                    _ => {}
                },
                NodeOrToken::Node(ref child) => {
                    self.format_expr(child);
                }
            }
        }
    }

    fn format_attr_access(&mut self, node: &SyntaxNode) {
        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Token(ref tok) => match tok.kind() {
                    SyntaxKind::DOT => self.write("."),
                    SyntaxKind::IDENT | SyntaxKind::NUMBER => self.write(tok.text()),
                    _ => {}
                },
                NodeOrToken::Node(ref child) => {
                    self.format_expr(child);
                }
            }
        }
    }

    fn format_index_expr(&mut self, node: &SyntaxNode) {
        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Token(ref tok) => match tok.kind() {
                    SyntaxKind::BRACKET_L => self.write("["),
                    SyntaxKind::BRACKET_R => self.write("]"),
                    _ => {}
                },
                NodeOrToken::Node(ref child) => {
                    self.format_expr(child);
                }
            }
        }
    }

    fn format_attr_splat(&mut self, node: &SyntaxNode) {
        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Token(ref tok) => match tok.kind() {
                    SyntaxKind::DOT => self.write("."),
                    SyntaxKind::STAR => self.write("*"),
                    _ => {}
                },
                NodeOrToken::Node(ref child) => match child.kind() {
                    SyntaxKind::SPLAT_BODY => self.format_splat_body(child),
                    _ => self.format_expr(child),
                },
            }
        }
    }

    fn format_index_splat(&mut self, node: &SyntaxNode) {
        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Token(ref tok) => match tok.kind() {
                    SyntaxKind::BRACKET_L => self.write("["),
                    SyntaxKind::BRACKET_R => self.write("]"),
                    SyntaxKind::STAR => self.write("*"),
                    _ => {}
                },
                NodeOrToken::Node(ref child) => match child.kind() {
                    SyntaxKind::SPLAT_BODY => self.format_splat_body(child),
                    _ => self.format_expr(child),
                },
            }
        }
    }

    fn format_splat_body(&mut self, node: &SyntaxNode) {
        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Token(ref tok) => match tok.kind() {
                    SyntaxKind::DOT => self.write("."),
                    SyntaxKind::IDENT => self.write(tok.text()),
                    SyntaxKind::BRACKET_L => self.write("["),
                    SyntaxKind::BRACKET_R => self.write("]"),
                    _ => {}
                },
                NodeOrToken::Node(ref child) => match child.kind() {
                    SyntaxKind::ATTR_ACCESS_EXPR => {
                        self.format_attr_access(child);
                    }
                    SyntaxKind::INDEX_EXPR => {
                        self.format_index_expr(child);
                    }
                    _ => self.format_expr(child),
                },
            }
        }
    }

    fn format_for_tuple(&mut self, node: &SyntaxNode) {
        self.write("[");
        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Token(ref tok) => match tok.kind() {
                    SyntaxKind::BRACKET_L | SyntaxKind::BRACKET_R => {}
                    _ => {}
                },
                NodeOrToken::Node(ref child) => match child.kind() {
                    SyntaxKind::FOR_INTRO => {
                        self.format_for_intro(child);
                        self.write(" ");
                    }
                    SyntaxKind::FOR_COND => {
                        self.write(" ");
                        self.format_for_cond(child);
                    }
                    _ => self.format_expr(child),
                },
            }
        }
        self.write("]");
    }

    fn format_for_object(&mut self, node: &SyntaxNode) {
        self.write("{");
        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Token(ref tok) => match tok.kind() {
                    SyntaxKind::BRACE_L | SyntaxKind::BRACE_R => {}
                    SyntaxKind::FAT_ARROW => self.write(" => "),
                    SyntaxKind::ELLIPSIS => self.write("..."),
                    _ => {}
                },
                NodeOrToken::Node(ref child) => match child.kind() {
                    SyntaxKind::FOR_INTRO => {
                        self.format_for_intro(child);
                        self.write(" ");
                    }
                    SyntaxKind::FOR_COND => {
                        self.write(" ");
                        self.format_for_cond(child);
                    }
                    _ => self.format_expr(child),
                },
            }
        }
        self.write("}");
    }

    fn format_for_intro(&mut self, node: &SyntaxNode) {
        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Token(ref tok) => match tok.kind() {
                    SyntaxKind::FOR_KW => {
                        self.write("for ");
                    }
                    SyntaxKind::IN_KW => self.write(" in "),
                    SyntaxKind::IDENT => {
                        self.write(tok.text());
                    }
                    SyntaxKind::COMMA => self.write(", "),
                    SyntaxKind::COLON => self.write(" :"),
                    _ => {}
                },
                NodeOrToken::Node(ref child) => {
                    self.format_expr(child);
                }
            }
        }
    }

    fn format_for_cond(&mut self, node: &SyntaxNode) {
        self.write("if ");
        for elem in node.children_with_tokens() {
            if is_trivia_element(&elem) {
                continue;
            }
            match elem {
                NodeOrToken::Token(ref tok) => match tok.kind() {
                    SyntaxKind::IF_KW => {} // already wrote "if "
                    _ => {}
                },
                NodeOrToken::Node(ref child) => {
                    self.format_expr(child);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrevItemKind {
    None,
    Attribute,
    Block,
    BlankLine,
    Comment,
}

enum BodyItem {
    Attribute {
        node: SyntaxNode,
        key_len: usize,
        multiline_value: bool,
    },
    Block {
        node: SyntaxNode,
    },
    BlankLine,
    Comment {
        text: String,
    },
}

struct AlignGroup {
    start: usize,
    end: usize,
    max_key_len: usize,
}

fn is_trivia(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE | SyntaxKind::LINE_COMMENT | SyntaxKind::BLOCK_COMMENT
    )
}

fn is_trivia_element(elem: &SyntaxElement) -> bool {
    match elem {
        NodeOrToken::Token(tok) => is_trivia(tok.kind()),
        _ => false,
    }
}

fn is_ident_like(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::IDENT
            | SyntaxKind::TRUE_KW
            | SyntaxKind::FALSE_KW
            | SyntaxKind::NULL_KW
            | SyntaxKind::FOR_KW
            | SyntaxKind::IN_KW
            | SyntaxKind::IF_KW
            | SyntaxKind::ELSE_KW
            | SyntaxKind::ENDIF_KW
            | SyntaxKind::ENDFOR_KW
    )
}

fn is_binary_op(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::PLUS
            | SyntaxKind::MINUS
            | SyntaxKind::STAR
            | SyntaxKind::SLASH
            | SyntaxKind::PERCENT
            | SyntaxKind::EQ_EQ
            | SyntaxKind::BANG_EQ
            | SyntaxKind::LT
            | SyntaxKind::LT_EQ
            | SyntaxKind::GT
            | SyntaxKind::GT_EQ
            | SyntaxKind::AMP_AMP
            | SyntaxKind::PIPE_PIPE
    )
}

fn node_contains_newline(node: &SyntaxNode) -> bool {
    for elem in node.children_with_tokens() {
        if let NodeOrToken::Token(ref tok) = elem {
            if tok.kind() == SyntaxKind::NEWLINE {
                return true;
            }
        }
    }
    false
}

fn node_contains_newline_recursive(node: &SyntaxNode) -> bool {
    for elem in node.descendants_with_tokens() {
        if let NodeOrToken::Token(ref tok) = elem {
            if tok.kind() == SyntaxKind::NEWLINE {
                return true;
            }
        }
    }
    false
}
