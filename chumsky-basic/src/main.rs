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
        .then_ignore(just('/'))
        .then(number(2).labelled("mm"))
        .then_ignore(just('/'))
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
    println!("Hello, world!");
}
