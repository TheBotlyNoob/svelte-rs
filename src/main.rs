#![warn(clippy::pedantic, clippy::nursery)]

mod js;

use itertools::Itertools;
use std::{collections::HashMap, ops::Range};

use ariadne::{sources, Color, Fmt, Label, Report, ReportKind};
use chumsky::prelude::*;

#[derive(Clone, Debug)]
enum Expr {
    JS(String), // TODO: parse JS
    Text(String),
    Element {
        name: String,
        attrs: HashMap<String, Expr>,
        children: Vec<Expr>,
        self_closing: bool,
    },
}

fn main() {
    let file = std::env::args()
        .nth(1)
        .unwrap();
    let file = &*Box::leak(file.into_boxed_str());
    let src = std::fs::read_to_string(file).unwrap();

    let (e, mut errs) = elem().parse_recovery(&*src);

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
    just('<')
        .ignore_then(text::ident())
        .then(
            text::whitespace()
                .ignore_then(attr())
                .repeated(),
        )
        .map(|(name, attrs)| Expr::Element {
            name,
            attrs: attrs.into_iter().collect(),
            children: vec![],
            self_closing: false,
        })
        .then_ignore(just('>'))
}

fn attr() -> impl Parser<char, (String, Expr), Error = Simple<char>> {
    text::ident()
        .then_ignore(just('=').padded())
        .then(
            just('{')
                .ignore_then(
                    take_until(just('}'))
                        .map(|(s, _)| s)
                        .collect(),
                )
                .map(Expr::JS)
                .or(just('"')
                    .ignore_then(take_until(just('"')))
                    .or(just('\'').ignore_then(take_until(just('\''))))
                    .map(|(s, _)| Expr::Text(s.into_iter().collect()))),
        )
}
