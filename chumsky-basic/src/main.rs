use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::prelude::*;

fn yyyy_mm_dd() -> impl Parser<char, (u32, u32, u32), Error = Simple<char>> {
    let number = |len| {
        text::digits(10)
            .validate(move |number: String, span, emit| {
                if number.len() != len {
                    emit(Simple::custom(
                        span,
                        format!("length of a number must be {}, but got {}", len, &number),
                    ))
                }
                number
            })
            .try_map(|number, span| {
                number.parse().map_err(|_| {
                    Simple::custom(span, format!("{} is an invalid u32 string", &number))
                })
            })
    };

    number(4)
        .labelled("yyyy")
        .then_ignore(just('/').labelled("between yyyy and mm"))
        .then(number(2).labelled("mm"))
        .then_ignore(just('/').labelled("betweel mm and dd"))
        .then(number(2).labelled("dd"))
        .map(|((y, m), d)| (y, m, d))
}

#[test]
fn test_yyyy_mm_dd() {
    assert_eq!(yyyy_mm_dd().parse("2020/03/19").unwrap(), (2020, 03, 19));
    assert!(yyyy_mm_dd().parse("20201/03/19").is_err());

    assert_eq!(
        yyyy_mm_dd().parse_recovery("20201/03/19").0,
        Some((20201, 03, 19))
    );
}

fn main() {
    for src in ["20221/03/19", "2021/june/10", "2022@10@10", "2022/"] {
        let (_, errs) = yyyy_mm_dd().parse_recovery(src);

        for e in errs {
            let message = match e.reason() {
                chumsky::error::SimpleReason::Unexpected
                | chumsky::error::SimpleReason::Unclosed { .. } => {
                    format!(
                        "{}{}, expected {}",
                        if e.found().is_some() {
                            "unexpected token"
                        } else {
                            "unexpected end of input"
                        },
                        if let Some(label) = e.label() {
                            format!(" while parsing {}", label.fg(Color::Green))
                        } else {
                            " something else".to_string()
                        },
                        if e.expected().count() == 0 {
                            "somemething else".to_string()
                        } else {
                            e.expected()
                                .map(|expected| match expected {
                                    Some(expected) => expected.to_string(),
                                    None => "end of input".to_string(),
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    )
                }
                chumsky::error::SimpleReason::Custom(msg) => msg.clone(),
            };

            Report::build(ReportKind::Error, (), e.span().start)
                .with_message(message)
                .with_label(Label::new(e.span()).with_message(match e.reason() {
                    chumsky::error::SimpleReason::Custom(msg) => msg.clone(),
                    _ => format!(
                            "Unexpected {}",
                            e.found()
                                .map(|c| format!("token {}", c.fg(Color::Red)))
                                .unwrap_or_else(|| "end of input".to_string())
                        ),
                }))
                .finish()
                .print(Source::from(src))
                .unwrap();
        }
    }
}
