#![warn(clippy::pedantic, clippy::nursery)]

mod js;

use itertools::Itertools;
use std::{collections::HashMap, ops::Range};

use ariadne::{sources, Color, Fmt, Label, Report, ReportKind};
use chumsky::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ElemTy {
    Wrapper,
    Component,
    HTML,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Expr {
    JS(String),
    Text(String),
    Elem {
        /// Is `None` when the [`ElemTy`] is [`Wrapper`](ElemTy::Wrapper)
        name: Option<String>,
        ty: ElemTy,
        attrs: HashMap<String, Expr>,
        children: Vec<Expr>,
    },
}

fn main() {
    let file = std::env::args()
        .nth(1)
        .unwrap();
    let file = &*Box::leak(file.into_boxed_str());
    let src = std::fs::read_to_string(file).unwrap();

    let (e, errs) = elem().parse_recovery(&*src);

    for e in errs
        .into_iter()
        .map(|e| e.map(|c| c.to_string()))
    {
        let report = Report::<(&str, Range<usize>)>::build(ReportKind::Error, file, e.span().start);

        let report = match e.reason() {
            chumsky::error::SimpleReason::Unclosed { span, delimiter } => report
                .with_message(format!(
                    "Unclosed delimiter {}",
                    delimiter.fg(Color::Yellow)
                ))
                .with_label(
                    Label::new((file, span.clone()))
                        .with_message(format!(
                            "Unclosed delimiter {}",
                            delimiter.fg(Color::Yellow)
                        ))
                        .with_color(Color::Yellow),
                )
                .with_label(
                    Label::new((file, e.span()))
                        .with_message(format!(
                            "Must be closed before this {}",
                            e.found()
                                .unwrap_or(&"end of file".to_string())
                                .fg(Color::Red)
                        ))
                        .with_color(Color::Red),
                ),
            chumsky::error::SimpleReason::Unexpected => report
                .with_message(format!(
                    "{}, expected {}",
                    if e.found().is_some() {
                        "Unexpected token in input"
                    } else {
                        "Unexpected end of input"
                    },
                    if e.expected().len() == 0 {
                        "something else".to_string()
                    } else {
                        e.expected()
                            .map(|expected| {
                                expected
                                    .as_deref()
                                    .unwrap_or("end of input")
                                    .to_string()
                                    .fg(Color::Green)
                            })
                            .format(", ")
                            .to_string()
                    }
                ))
                .with_label(
                    Label::new((file, e.span()))
                        .with_message(format!(
                            "Unexpected token {}",
                            e.found()
                                .unwrap_or(&"end of file".to_string())
                                .fg(Color::Red)
                        ))
                        .with_color(Color::Red),
                ),
            chumsky::error::SimpleReason::Custom(msg) => report
                .with_message(msg)
                .with_label(
                    Label::new((file, e.span()))
                        .with_message(format!("{}", msg.fg(Color::Red)))
                        .with_color(Color::Red),
                ),
        };

        let srcs = sources([(file, &*src)]);

        report
            .finish()
            .eprint(srcs)
            .unwrap();
    }

    println!("{e:#?}");
}

fn elem() -> impl Parser<char, Expr, Error = Simple<char>> {
    choice((
        just('/')
            .or_not()
            .then(text::ident().padded())
            .then(
                attr()
                    .padded()
                    .repeated()
                    .collect(),
            )
            .then(just('/').or_not())
            .map(|(((closing, name), attrs), self_closing)| Expr::Elem {
                ty: if name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    ElemTy::Component
                } else {
                    ElemTy::HTML
                },
                name: Some(name),
                attrs,
                children: if closing
                    .zip(self_closing)
                    .is_none()
                {
                    // we're not a closing tag
                    vec![]
                } else {
                    vec![]
                },
            })
            .delimited_by(just('<'), just('>')),
        filter(|&c: &char| c != '<')
            .repeated()
            .collect()
            .map(Expr::Text),
    ))
}

fn attr() -> impl Parser<char, (String, Expr), Error = Simple<char>> {
    // helper fn for delimiters
    // TODO: use `delimited_by()` instead
    fn p(s: char, e: char, f: fn(String) -> Expr) -> impl Parser<char, Expr, Error = Simple<char>> {
        just(s)
            .ignore_then(take_until(just(e)))
            .map(move |(s, _)| f(s.into_iter().collect()))
    }

    text::ident()
        .then_ignore(just('=').padded())
        .then(choice((
            p('{', '}', Expr::JS),
            p('"', '"', Expr::Text),
            p('\'', '\'', Expr::Text),
        )))
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! wrapper {
        ($n:ident, $out:ty) => {
            use super::*;
            fn $n(s: impl AsRef<str>) -> Result<$out, Vec<Simple<char>>> {
                super::$n().parse(s.as_ref())
            }
        };
    }

    mod attr {
        use chumsky::error::SimpleReason;

        wrapper!(attr, (String, Expr));

        #[test]
        fn text() {
            let ex = Ok(("a".into(), Expr::Text("b".into())));
            assert_eq!(attr("a=\"b\""), ex);
            assert_eq!(attr("a='b'"), ex);
        }

        #[test]
        fn js() {
            assert_eq!(attr("a={b()}"), Ok(("a".into(), Expr::JS("b()".into()))));
        }

        #[test]
        fn without_quotes() {
            let a = attr("a=b").expect_err("no quotes or braces");
            let a = a.get(0).expect("one error");

            assert_eq!(*a.reason(), SimpleReason::Unexpected);
            assert_eq!(a.span(), 2..3);
        }
    }

    mod elem {
        wrapper!(elem, Expr);

        #[test]
        fn text() {
            assert_eq!(elem("text<"), Ok(Expr::Text("text".into())))
        }

        #[test]
        fn self_closing() {
            assert_eq!(
                elem("<e />"),
                Ok(Expr::Elem {
                    name: Some("e".into()),
                    ty: ElemTy::HTML,
                    attrs: HashMap::new(),
                    children: vec![],
                })
            );
        }

        #[test]
        #[ignore = "not yet implemented"]
        fn children() {
            assert_eq!(
                elem("<e>text</e>"),
                Ok(Expr::Elem {
                    name: Some("e".into()),
                    ty: ElemTy::HTML,
                    attrs: HashMap::new(),
                    children: vec![Expr::Text("text".into())]
                })
            )
        }

        #[test]
        fn component() {
            assert_eq!(
                elem("<E>"),
                Ok(Expr::Elem {
                    name: Some("E".into()),
                    ty: ElemTy::Component,
                    attrs: HashMap::new(),
                    children: vec![],
                })
            )
        }
    }
}
