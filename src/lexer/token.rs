use std::fmt::Display;

/// Use to tokenize the input text
#[derive(logos::Logos, Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenKind {
    #[regex("[ \n\r]+")]
    Whitespace,

    #[token("create", ignore(case))]
    CreateKw,

    #[token("procedure", ignore(case))]
    ProcedureKw,

    #[token("or replace", ignore(case))]
    OrReplaceKw,

    #[token("begin", ignore(case))]
    BeginKw,

    #[token("is", ignore(case))]
    IsKw,

    #[token("end", ignore(case))]
    EndKw,

    #[token("out", ignore(case))]
    OutKw,

    #[token("in", ignore(case))]
    InKw,

    #[regex("[A-Za-z0-9_][A-Za-z0-9_$.]*")]
    Ident,

    #[regex("'[A-Za-z0-9 ]+'")]
    QuotedLiteral,

    #[token(",")]
    Comma,

    #[token(".")]
    Dot,

    #[token(";")]
    SemiColon,

    #[token(":=")]
    Assign,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("%")]
    Percentage,

    #[regex("--.*")]
    Comment,

    #[error]
    Error,
}

impl TokenKind {
    pub fn is_trivia(self) -> bool {
        matches!(self, Self::Whitespace | Self::Comment)
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::TokenKind;
    use crate::Lexer;

    fn check(input: &str, kind: TokenKind) {
        let mut lexer = Lexer::new(input);
        let token = lexer.next().unwrap();
        assert_eq!(token.kind, kind);
        assert_eq!(token.text, input);
    }

    #[test]
    fn lex_spaces_and_newlines() {
        check("  \n", TokenKind::Whitespace);
    }

    #[test]
    fn lex_ident() {
        check("hello", TokenKind::Ident);
    }
}
