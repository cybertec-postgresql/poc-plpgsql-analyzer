use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use rowan::{NodeOrToken, GreenToken, GreenNode};

/// Examples
/// * https://blog.kiranshila.com/blog/easy_cst.md
/// * https://arzg.github.io/lang/10/
/// * https://github.com/rust-analyzer/rowan/blob/master/examples/s_expressions.rs
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, FromPrimitive, ToPrimitive)]
#[repr(u16)]
pub enum SyntaxKind {
    LeftParen = 0,
    RightParen,
    Comment,
    Whitespace,
    Keyword,
    Ident,
    Comma,
    Procedure,
    ProcedureStart,
    ProcedureParam,
    ProcedureParams,
    ProcedureBody,
    Root,
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        rowan::SyntaxKind(kind.to_u16().unwrap())
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum SqlProcedureLang {}

impl rowan::Language for SqlProcedureLang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        Self::Kind::from_u16(raw.0).unwrap()
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind.to_u16().unwrap())
    }
}

pub type SyntaxNode = rowan::SyntaxNode<SqlProcedureLang>;
pub type SyntaxElement = NodeOrToken<GreenNode, GreenToken>;

/// Creates a new leaf node.
pub fn leaf(kind: SyntaxKind, input: &str) -> SyntaxElement {
    NodeOrToken::Token(GreenToken::new(kind.into(), input))
}

/// Creates a new collection of nodes.
pub fn node(kind: SyntaxKind, children: Vec<SyntaxElement>) -> SyntaxElement {
    NodeOrToken::Node(GreenNode::new(kind.into(), children))
}
