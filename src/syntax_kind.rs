use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::ToPrimitive;
use rowan::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, FromPrimitive, ToPrimitive)]
#[repr(u16)]
#[allow(non_camel_case_types)]
pub enum SyntaxKind {
    // === Trivia tokens ===
    WHITESPACE = 0,
    NEWLINE,
    LINE_COMMENT,
    BLOCK_COMMENT,

    // === Literal tokens ===
    NUMBER,
    TRUE_KW,
    FALSE_KW,
    NULL_KW,

    // === String tokens ===
    STRING_LIT,
    HEREDOC_ANCHOR,
    HEREDOC_CONTENT,
    STRING_FRAGMENT,
    ESCAPE_SEQUENCE,
    DOLLAR_OPEN,
    PERCENT_OPEN,
    TEMPLATE_CLOSE,

    // === Identifier / keyword tokens ===
    IDENT,
    FOR_KW,
    IN_KW,
    IF_KW,
    ELSE_KW,
    ENDIF_KW,
    ENDFOR_KW,

    // === Operator tokens ===
    PLUS,
    MINUS,
    STAR,
    SLASH,
    PERCENT,
    EQ_EQ,
    BANG_EQ,
    LT,
    LT_EQ,
    GT,
    GT_EQ,
    AMP_AMP,
    PIPE_PIPE,
    BANG,

    // === Punctuation tokens ===
    EQ,
    FAT_ARROW,
    PAREN_L,
    PAREN_R,
    BRACE_L,
    BRACE_R,
    BRACKET_L,
    BRACKET_R,
    COMMA,
    DOT,
    COLON,
    QUESTION,
    ELLIPSIS,
    HEREDOC_OPEN,
    TILDE,

    // === Special tokens ===
    ERROR_TOKEN,
    QUOTE,

    // === Node kinds ===
    SOURCE_FILE,
    BODY,

    // Structural nodes
    ATTRIBUTE,
    BLOCK,
    BLOCK_LABEL,

    // Expression nodes
    LITERAL_EXPR,
    STRING_EXPR,
    HEREDOC_EXPR,
    TEMPLATE_INTERPOLATION,
    TEMPLATE_DIRECTIVE,
    VARIABLE_EXPR,
    FUNCTION_CALL,
    ARG_LIST,
    PAREN_EXPR,
    TUPLE_EXPR,
    OBJECT_EXPR,
    OBJECT_ELEM,

    // Operation nodes
    UNARY_EXPR,
    BINARY_EXPR,
    CONDITIONAL_EXPR,

    // Access nodes
    INDEX_EXPR,
    ATTR_ACCESS_EXPR,
    ATTR_SPLAT_EXPR,
    INDEX_SPLAT_EXPR,
    SPLAT_BODY,

    // For-expression nodes
    FOR_TUPLE_EXPR,
    FOR_OBJECT_EXPR,
    FOR_INTRO,
    FOR_COND,

    // Error node
    ERROR,
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        rowan::SyntaxKind(kind.to_u16().unwrap())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HclLang;

impl Language for HclLang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        num_traits::FromPrimitive::from_u16(raw.0).unwrap()
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind.to_u16().unwrap())
    }
}

pub type SyntaxNode = rowan::SyntaxNode<HclLang>;
pub type SyntaxToken = rowan::SyntaxToken<HclLang>;
pub type SyntaxElement = rowan::SyntaxElement<HclLang>;
