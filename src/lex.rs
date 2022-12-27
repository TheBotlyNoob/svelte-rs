#[derive(Debug, Clone, Copy, PartialEq, Eq, logos::Logos)]
pub enum Token<'a> {
    #[token("<")]
    LessThan,
    #[token(">")]
    GreaterThan,
    #[token("=")]
    Equal,
    #[token("+")]
    Plus,
    #[token("-")]
    Dash,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token(":")]
    Colon,
    #[token("!")]
    Bang,
    #[token("#")]
    Hash,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident(&'a str),
    #[regex(r"[0-9]+")]
    Number,

    #[regex(r"[\n;]")]
    Eol,

    #[error]
    #[regex(r"[ \t]", logos::skip)]
    Error,
}

#[cfg(test)]
mod test {
    use logos::Logos;

    use super::*;

    #[test]
    fn all() {
        let lexer = Token::lexer("<>=+-*/(){}[],.;:!#a 1");
        let tokens: Vec<Token> = lexer.collect();
        assert_eq!(
            tokens,
            vec![
                Token::LessThan,
                Token::GreaterThan,
                Token::Equal,
                Token::Plus,
                Token::Dash,
                Token::Star,
                Token::Slash,
                Token::LeftParen,
                Token::RightParen,
                Token::LeftBrace,
                Token::RightBrace,
                Token::LeftBracket,
                Token::RightBracket,
                Token::Comma,
                Token::Dot,
                Token::Eol,
                Token::Colon,
                Token::Bang,
                Token::Hash,
                Token::Ident("a"),
                Token::Number,
            ]
        );
    }

    #[test]
    fn whitespace() {
        let lexer = Token::lexer("<div  ></div>");
        let tokens: Vec<Token> = lexer.collect();
        assert_eq!(
            tokens,
            vec![
                Token::LessThan,
                Token::Ident("div"),
                Token::GreaterThan,
                Token::LessThan,
                Token::Slash,
                Token::Ident("div"),
                Token::GreaterThan,
            ]
        );
    }
}
