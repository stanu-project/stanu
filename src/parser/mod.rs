pub mod body;
pub mod expr;
pub mod template;

use rowan::GreenNode;
use rowan::GreenNodeBuilder;

use crate::error::ParseError;
use crate::lexer::Token;
use crate::syntax_kind::SyntaxKind;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    builder: GreenNodeBuilder<'static>,
    errors: Vec<ParseError>,
    source_len: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, source: &str) -> Self {
        Self {
            tokens,
            pos: 0,
            builder: GreenNodeBuilder::new(),
            errors: Vec::new(),
            source_len: source.len(),
        }
    }

    pub fn parse(mut self) -> (GreenNode, Vec<ParseError>) {
        body::parse_source_file(&mut self);
        let green = self.builder.finish();
        (green, self.errors)
    }

    // ── Navigation ───────────────────────────────────────────────

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek(&self) -> Option<SyntaxKind> {
        self.current().map(|t| t.kind)
    }

    /// Peek past trivia to find the next "meaningful" token kind.
    fn peek_non_trivia(&self) -> Option<SyntaxKind> {
        let mut i = self.pos;
        while i < self.tokens.len() {
            let kind = self.tokens[i].kind;
            if !Self::is_trivia(kind) {
                return Some(kind);
            }
            i += 1;
        }
        None
    }

    /// Look ahead past trivia by `n` meaningful tokens (0-based).
    fn peek_non_trivia_nth(&self, n: usize) -> Option<SyntaxKind> {
        let mut i = self.pos;
        let mut count = 0;
        while i < self.tokens.len() {
            let kind = self.tokens[i].kind;
            if !Self::is_trivia(kind) {
                if count == n {
                    return Some(kind);
                }
                count += 1;
            }
            i += 1;
        }
        None
    }

    fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn is_trivia(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::WHITESPACE
                | SyntaxKind::NEWLINE
                | SyntaxKind::LINE_COMMENT
                | SyntaxKind::BLOCK_COMMENT
        )
    }

    // ── Token consumption ────────────────────────────────────────

    fn bump(&mut self) {
        if let Some(token) = self.tokens.get(self.pos) {
            self.builder
                .token(token.kind.into(), &token.text);
            self.pos += 1;
        }
    }

    fn eat(&mut self, kind: SyntaxKind) -> bool {
        if self.peek() == Some(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: SyntaxKind) -> bool {
        if self.eat(kind) {
            return true;
        }
        let offset = self.current_offset();
        let found = self
            .peek()
            .map(|k| format!("{:?}", k))
            .unwrap_or_else(|| "EOF".to_string());
        self.errors.push(ParseError::new(
            format!("expected {:?}, found {}", kind, found),
            offset,
        ));
        false
    }

    fn skip_trivia(&mut self) {
        while let Some(kind) = self.peek() {
            if Self::is_trivia(kind) {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn current_offset(&self) -> usize {
        if self.current().is_some() {
            let mut offset = 0;
            for t in &self.tokens[..self.pos] {
                offset += t.text.len();
            }
            offset
        } else {
            self.source_len
        }
    }

    // ── Node building ────────────────────────────────────────────

    fn start_node(&mut self, kind: SyntaxKind) {
        self.builder.start_node(kind.into());
    }

    fn finish_node(&mut self) {
        self.builder.finish_node();
    }

    fn checkpoint(&mut self) -> rowan::Checkpoint {
        self.builder.checkpoint()
    }

    fn start_node_at(&mut self, checkpoint: rowan::Checkpoint, kind: SyntaxKind) {
        self.builder.start_node_at(checkpoint, kind.into());
    }
}
