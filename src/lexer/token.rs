/// Use to tokenize the input text
#[derive(logos::Logos, Debug, Copy, Clone, PartialEq)]
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

    #[regex("[A-Za-z0-9_][A-Za-z0-9_$.]*")]
    Ident,

    #[token(",")]
    Comma,

    #[token(".")]
    Dot,

    #[token(";")]
    SemiColon,

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

#[cfg(test)]
mod tests {
    use crate::Lexer;
    use super::TokenKind;

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
