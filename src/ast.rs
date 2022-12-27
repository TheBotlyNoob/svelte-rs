use std::iter::Peekable;

use crate::lex::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    OpenElement {
        name: String,
        attributes: Vec<Attribute>,
    },
    CloseElement {
        name: String,
    },
    Text(String),
    Comment(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

impl Node {
    pub fn from_tokens<'a>(lexer: impl Iterator<Item = Token<'a>>) -> Option<Self> {
        let mut tokens = lexer.peekable();
        let token = *tokens.peek()?;

        macro_rules! p {
            ($($token:pat => $body:expr),*) => {
                match token {
                    $($token => {
                        println!("matched {token:?}");
                        tokens.next();
                        #[allow(unused_variables)]
                        let token = tokens.peek()?;
                        println!("next token: {token:?}");
                        $body
                    },)*
                    _ => return None,
                }
            };
        }

        p! {
            Token::LessThan => p! {
                Token::Ident(name) => {
                    let attributes = Self::parse_attributes(&mut tokens);
                    Some(Node::OpenElement { name: name.to_string(), attributes })
                },
                Token::Bang => p! {
                    Token::Dash => p! {
                        Token::Dash => p! {
                            Token::Ident(name) => {
                                Some(Node::CloseElement { name: name.to_string() })
                            }
                        }
                    }
                }
            }
        }
    }

    fn parse_attributes<'a>(
        tokens: &mut Peekable<impl Iterator<Item = Token<'a>>>,
    ) -> Vec<Attribute> {
        vec![]
    }
}

#[cfg(test)]
mod test {
    use logos::Logos;

    use super::*;

    #[test]
    fn open_elem() {
        let lexer = Token::lexer("<a>");
        let mut p = lexer.peekable();
        let t = *p.peek().unwrap();
        println!("{:?}", t);
        p.next();
        let t = *p.peek().unwrap();
        println!("{:?}", t);
        match t {
            Token::Ident(name) => {
                println!("matched ident: {}", name);
            }
            _ => panic!("expected ident"),
        }

        // let node = Node::from_tokens(lexer).unwrap();
        // assert_eq!(
        //     node,
        //     Node::OpenElement {
        //         name: "a".to_string(),
        //         attributes: vec![]
        //     }
        // );
    }
}
