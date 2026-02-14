use crate::syntax_kind::SyntaxKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: SyntaxKind,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    StringTemplate,
    HeredocTemplate,
    Interpolation,
    Directive,
}

#[derive(Debug, Clone)]
struct ModeEntry {
    mode: Mode,
    brace_depth: usize,
    heredoc_anchor: String,
    heredoc_indent: bool,
}

impl ModeEntry {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            brace_depth: 0,
            heredoc_anchor: String::new(),
            heredoc_indent: false,
        }
    }

    fn heredoc(anchor: String, indent: bool) -> Self {
        Self {
            mode: Mode::HeredocTemplate,
            brace_depth: 0,
            heredoc_anchor: anchor,
            heredoc_indent: indent,
        }
    }
}

pub struct Lexer<'a> {
    source: &'a str,
    pos: usize,
    mode_stack: Vec<ModeEntry>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            pos: 0,
            mode_stack: vec![ModeEntry::new(Mode::Normal)],
        }
    }

    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while self.pos < self.source.len() {
            let token = self.next_token();
            tokens.push(token);
        }
        tokens
    }

    fn current_mode(&self) -> Mode {
        self.mode_stack.last().map(|e| e.mode).unwrap_or(Mode::Normal)
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn peek_char_at(&self, offset: usize) -> Option<char> {
        self.source[self.pos + offset..].chars().next()
    }

    fn remaining(&self) -> &str {
        &self.source[self.pos..]
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.pos..].chars().next().unwrap();
        self.pos += c.len_utf8();
        c
    }

    fn advance_n(&mut self, n: usize) {
        for _ in 0..n {
            if self.pos < self.source.len() {
                let c = self.source[self.pos..].chars().next().unwrap();
                self.pos += c.len_utf8();
            }
        }
    }

    fn advance_bytes(&mut self, n: usize) {
        self.pos += n;
    }

    fn next_token(&mut self) -> Token {
        match self.current_mode() {
            Mode::Normal | Mode::Interpolation | Mode::Directive => self.lex_normal(),
            Mode::StringTemplate => self.lex_string_template(),
            Mode::HeredocTemplate => self.lex_heredoc_template(),
        }
    }

    // ── Normal mode ──────────────────────────────────────────────

    fn lex_normal(&mut self) -> Token {
        let start = self.pos;
        let c = self.peek_char().unwrap();

        // Whitespace (not newlines)
        if c == ' ' || c == '\t' || c == '\r' {
            return self.lex_whitespace(start);
        }

        // Newlines
        if c == '\n' {
            self.advance();
            return self.make_token(SyntaxKind::NEWLINE, start);
        }

        // Line comments: # or //
        if c == '#' {
            return self.lex_line_comment(start);
        }
        if c == '/' && self.peek_char_at(1) == Some('/') {
            return self.lex_line_comment(start);
        }

        // Block comments: /* ... */
        if c == '/' && self.peek_char_at(1) == Some('*') {
            return self.lex_block_comment(start);
        }

        // Numbers
        if c.is_ascii_digit() {
            return self.lex_number(start);
        }

        // Identifiers and keywords
        if c.is_ascii_alphabetic() || c == '_' {
            return self.lex_ident(start);
        }

        // Strings
        if c == '"' {
            self.advance(); // consume opening quote
            self.mode_stack.push(ModeEntry::new(Mode::StringTemplate));
            return self.make_token(SyntaxKind::QUOTE, start);
        }

        // Heredoc
        if c == '<' && self.peek_char_at(1) == Some('<') {
            return self.lex_heredoc_open(start);
        }

        // Operators and punctuation
        self.lex_operator(start)
    }

    fn lex_whitespace(&mut self, start: usize) -> Token {
        while self.pos < self.source.len() {
            match self.peek_char() {
                Some(' ' | '\t' | '\r') => { self.advance(); }
                _ => break,
            }
        }
        self.make_token(SyntaxKind::WHITESPACE, start)
    }

    fn lex_line_comment(&mut self, start: usize) -> Token {
        while self.pos < self.source.len() {
            if self.peek_char() == Some('\n') {
                break;
            }
            self.advance();
        }
        self.make_token(SyntaxKind::LINE_COMMENT, start)
    }

    fn lex_block_comment(&mut self, start: usize) -> Token {
        self.advance_n(2); // consume /*
        let mut depth = 1;
        while self.pos < self.source.len() && depth > 0 {
            if self.remaining().starts_with("/*") {
                depth += 1;
                self.advance_n(2);
            } else if self.remaining().starts_with("*/") {
                depth -= 1;
                self.advance_n(2);
            } else {
                self.advance();
            }
        }
        self.make_token(SyntaxKind::BLOCK_COMMENT, start)
    }

    fn lex_number(&mut self, start: usize) -> Token {
        // Integer part
        while self.pos < self.source.len() {
            match self.peek_char() {
                Some(c) if c.is_ascii_digit() => { self.advance(); }
                _ => break,
            }
        }
        // Fractional part
        if self.peek_char() == Some('.') && self.peek_char_at(1).is_some_and(|c| c.is_ascii_digit()) {
            self.advance(); // consume .
            while self.pos < self.source.len() {
                match self.peek_char() {
                    Some(c) if c.is_ascii_digit() => { self.advance(); }
                    _ => break,
                }
            }
        }
        // Exponent part
        if matches!(self.peek_char(), Some('e' | 'E')) {
            self.advance();
            if matches!(self.peek_char(), Some('+' | '-')) {
                self.advance();
            }
            while self.pos < self.source.len() {
                match self.peek_char() {
                    Some(c) if c.is_ascii_digit() => { self.advance(); }
                    _ => break,
                }
            }
        }
        self.make_token(SyntaxKind::NUMBER, start)
    }

    fn lex_ident(&mut self, start: usize) -> Token {
        while self.pos < self.source.len() {
            match self.peek_char() {
                Some(c) if c.is_ascii_alphanumeric() || c == '_' || c == '-' => {
                    self.advance();
                }
                _ => break,
            }
        }
        let text = &self.source[start..self.pos];
        let kind = match text {
            "true" => SyntaxKind::TRUE_KW,
            "false" => SyntaxKind::FALSE_KW,
            "null" => SyntaxKind::NULL_KW,
            "for" => SyntaxKind::FOR_KW,
            "in" => SyntaxKind::IN_KW,
            "if" => SyntaxKind::IF_KW,
            "else" => SyntaxKind::ELSE_KW,
            "endif" => SyntaxKind::ENDIF_KW,
            "endfor" => SyntaxKind::ENDFOR_KW,
            _ => SyntaxKind::IDENT,
        };
        self.make_token(kind, start)
    }

    fn lex_heredoc_open(&mut self, start: usize) -> Token {
        self.advance_n(2); // consume <<
        let indent = if self.peek_char() == Some('-') {
            self.advance();
            true
        } else {
            false
        };
        // Read anchor identifier
        let anchor_start = self.pos;
        while self.pos < self.source.len() {
            match self.peek_char() {
                Some(c) if c.is_ascii_alphanumeric() || c == '_' => { self.advance(); }
                _ => break,
            }
        }
        let anchor = self.source[anchor_start..self.pos].to_string();
        // Consume the newline after the anchor
        if self.peek_char() == Some('\r') {
            self.advance();
        }
        if self.peek_char() == Some('\n') {
            self.advance();
        }
        self.mode_stack.push(ModeEntry::heredoc(anchor, indent));
        self.make_token(SyntaxKind::HEREDOC_OPEN, start)
    }

    fn lex_operator(&mut self, start: usize) -> Token {
        let c = self.advance();
        match c {
            '=' => {
                if self.peek_char() == Some('=') {
                    self.advance();
                    self.make_token(SyntaxKind::EQ_EQ, start)
                } else if self.peek_char() == Some('>') {
                    self.advance();
                    self.make_token(SyntaxKind::FAT_ARROW, start)
                } else {
                    self.make_token(SyntaxKind::EQ, start)
                }
            }
            '!' => {
                if self.peek_char() == Some('=') {
                    self.advance();
                    self.make_token(SyntaxKind::BANG_EQ, start)
                } else {
                    self.make_token(SyntaxKind::BANG, start)
                }
            }
            '<' => {
                if self.peek_char() == Some('=') {
                    self.advance();
                    self.make_token(SyntaxKind::LT_EQ, start)
                } else {
                    self.make_token(SyntaxKind::LT, start)
                }
            }
            '>' => {
                if self.peek_char() == Some('=') {
                    self.advance();
                    self.make_token(SyntaxKind::GT_EQ, start)
                } else {
                    self.make_token(SyntaxKind::GT, start)
                }
            }
            '&' if self.peek_char() == Some('&') => {
                self.advance();
                self.make_token(SyntaxKind::AMP_AMP, start)
            }
            '|' if self.peek_char() == Some('|') => {
                self.advance();
                self.make_token(SyntaxKind::PIPE_PIPE, start)
            }
            '+' => self.make_token(SyntaxKind::PLUS, start),
            '-' => self.make_token(SyntaxKind::MINUS, start),
            '*' => self.make_token(SyntaxKind::STAR, start),
            '/' => self.make_token(SyntaxKind::SLASH, start),
            '%' => {
                if self.peek_char() == Some('{') {
                    // This shouldn't happen in normal mode normally,
                    // but handle it for robustness
                    self.make_token(SyntaxKind::PERCENT, start)
                } else {
                    self.make_token(SyntaxKind::PERCENT, start)
                }
            }
            '(' => self.make_token(SyntaxKind::PAREN_L, start),
            ')' => self.make_token(SyntaxKind::PAREN_R, start),
            '{' => {
                if let Some(entry) = self.mode_stack.last_mut() {
                    if entry.mode == Mode::Interpolation || entry.mode == Mode::Directive {
                        entry.brace_depth += 1;
                    }
                }
                self.make_token(SyntaxKind::BRACE_L, start)
            }
            '}' => {
                // Check if this closes an interpolation/directive
                let should_pop = if let Some(entry) = self.mode_stack.last_mut() {
                    if (entry.mode == Mode::Interpolation || entry.mode == Mode::Directive)
                        && entry.brace_depth == 0
                    {
                        true
                    } else {
                        if entry.mode == Mode::Interpolation || entry.mode == Mode::Directive {
                            entry.brace_depth -= 1;
                        }
                        false
                    }
                } else {
                    false
                };
                if should_pop {
                    self.mode_stack.pop();
                    self.make_token(SyntaxKind::TEMPLATE_CLOSE, start)
                } else {
                    self.make_token(SyntaxKind::BRACE_R, start)
                }
            }
            '[' => self.make_token(SyntaxKind::BRACKET_L, start),
            ']' => self.make_token(SyntaxKind::BRACKET_R, start),
            ',' => self.make_token(SyntaxKind::COMMA, start),
            '.' => {
                if self.remaining().starts_with("..") {
                    self.advance_n(2);
                    self.make_token(SyntaxKind::ELLIPSIS, start)
                } else {
                    self.make_token(SyntaxKind::DOT, start)
                }
            }
            ':' => self.make_token(SyntaxKind::COLON, start),
            '?' => self.make_token(SyntaxKind::QUESTION, start),
            '~' => self.make_token(SyntaxKind::TILDE, start),
            _ => self.make_token(SyntaxKind::ERROR_TOKEN, start),
        }
    }

    // ── String template mode ─────────────────────────────────────

    fn lex_string_template(&mut self) -> Token {
        let start = self.pos;
        let c = self.peek_char().unwrap();

        match c {
            '"' => {
                self.advance();
                self.mode_stack.pop(); // back to previous mode
                self.make_token(SyntaxKind::QUOTE, start)
            }
            '$' if self.peek_char_at(1) == Some('{') => {
                self.advance_n(2);
                self.mode_stack.push(ModeEntry::new(Mode::Interpolation));
                self.make_token(SyntaxKind::DOLLAR_OPEN, start)
            }
            '%' if self.peek_char_at(1) == Some('{') => {
                self.advance_n(2);
                self.mode_stack.push(ModeEntry::new(Mode::Directive));
                self.make_token(SyntaxKind::PERCENT_OPEN, start)
            }
            '\\' => self.lex_escape_sequence(start),
            _ => self.lex_string_fragment(start),
        }
    }

    fn lex_escape_sequence(&mut self, start: usize) -> Token {
        self.advance(); // consume backslash
        if self.pos >= self.source.len() {
            return self.make_token(SyntaxKind::ERROR_TOKEN, start);
        }
        let c = self.advance();
        match c {
            '"' | '\\' | 'n' | 'r' | 't' | 'a' | 'b' | 'f' | 'v' => {}
            'u' => {
                // \uXXXX
                for _ in 0..4 {
                    if self.pos < self.source.len()
                        && self.peek_char().is_some_and(|c| c.is_ascii_hexdigit())
                    {
                        self.advance();
                    }
                }
            }
            'U' => {
                // \UNNNNNNNN
                for _ in 0..8 {
                    if self.pos < self.source.len()
                        && self.peek_char().is_some_and(|c| c.is_ascii_hexdigit())
                    {
                        self.advance();
                    }
                }
            }
            '$' | '%' => {
                // Escaped template markers
            }
            _ => {
                // Unknown escape - still consume it
            }
        }
        self.make_token(SyntaxKind::ESCAPE_SEQUENCE, start)
    }

    fn lex_string_fragment(&mut self, start: usize) -> Token {
        while self.pos < self.source.len() {
            match self.peek_char() {
                Some('"') | Some('\\') => break,
                Some('$') if self.peek_char_at(1) == Some('{') => break,
                Some('%') if self.peek_char_at(1) == Some('{') => break,
                Some(_) => { self.advance(); }
                None => break,
            }
        }
        self.make_token(SyntaxKind::STRING_FRAGMENT, start)
    }

    // ── Heredoc template mode ────────────────────────────────────

    fn lex_heredoc_template(&mut self) -> Token {
        let start = self.pos;

        // Check if we're at the closing anchor (possibly indented)
        if self.is_at_heredoc_close() {
            let anchor_len = self.mode_stack.last().unwrap().heredoc_anchor.len();
            let indent = self.mode_stack.last().unwrap().heredoc_indent;
            // Consume optional leading whitespace for indented heredocs
            if indent {
                while self.pos < self.source.len() {
                    match self.peek_char() {
                        Some(' ' | '\t') => { self.advance(); }
                        _ => break,
                    }
                }
            }
            self.advance_bytes(anchor_len);
            self.mode_stack.pop();
            return self.make_token(SyntaxKind::HEREDOC_ANCHOR, start);
        }

        let c = self.peek_char().unwrap();

        match c {
            '$' if self.peek_char_at(1) == Some('{') => {
                self.advance_n(2);
                self.mode_stack.push(ModeEntry::new(Mode::Interpolation));
                self.make_token(SyntaxKind::DOLLAR_OPEN, start)
            }
            '%' if self.peek_char_at(1) == Some('{') => {
                self.advance_n(2);
                self.mode_stack.push(ModeEntry::new(Mode::Directive));
                self.make_token(SyntaxKind::PERCENT_OPEN, start)
            }
            _ => self.lex_heredoc_content(start),
        }
    }

    fn is_at_heredoc_close(&self) -> bool {
        let entry = match self.mode_stack.last() {
            Some(e) if e.mode == Mode::HeredocTemplate => e,
            _ => return false,
        };
        let remaining = self.remaining();

        // Determine the start of content (skip leading whitespace for indented heredocs)
        let content = if entry.heredoc_indent {
            remaining.trim_start_matches([' ', '\t'])
        } else {
            remaining
        };

        // Check if at start of line (pos == 0 or previous char is \n)
        let at_line_start = self.pos == 0 || self.source.as_bytes()[self.pos - 1] == b'\n';
        if !at_line_start {
            return false;
        }

        // Check if content starts with anchor followed by newline or EOF
        if content.starts_with(&entry.heredoc_anchor) {
            let after_anchor = &content[entry.heredoc_anchor.len()..];
            after_anchor.is_empty()
                || after_anchor.starts_with('\n')
                || after_anchor.starts_with('\r')
        } else {
            false
        }
    }

    fn lex_heredoc_content(&mut self, start: usize) -> Token {
        // Consume content until we hit a template marker or end of line
        while self.pos < self.source.len() {
            match self.peek_char() {
                Some('$') if self.peek_char_at(1) == Some('{') => break,
                Some('%') if self.peek_char_at(1) == Some('{') => break,
                Some('\n') => {
                    self.advance(); // consume the newline
                    // Check if next line is closing anchor
                    if self.is_at_heredoc_close() {
                        break;
                    }
                }
                Some(_) => { self.advance(); }
                None => break,
            }
        }
        self.make_token(SyntaxKind::HEREDOC_CONTENT, start)
    }

    fn make_token(&self, kind: SyntaxKind, start: usize) -> Token {
        Token {
            kind,
            text: self.source[start..self.pos].to_string(),
        }
    }
}
