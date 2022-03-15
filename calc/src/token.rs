use chumsky::prelude::*;
use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    With,
    Ident(String),
    Number(u32),
    Comma,
    Colon,
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Error(char),
}

pub fn lexer() -> impl Parser<char, Vec<(Token, Range<usize>)>, Error = Simple<char>> {
    let with = just("with").to(Token::With);
    let ident = filter(|c: &char| (*c >= 'a' && *c <= 'z') || (*c >= 'A' && *c <= 'Z'))
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(Token::Ident);
    let number = text::int(10).from_str().unwrapped().map(Token::Number);
    let comma = just(',').to(Token::Comma);
    let colon = just(':').to(Token::Colon);
    let plus = just('+').to(Token::Plus);
    let minus = just('-').to(Token::Minus);
    let star = just('*').to(Token::Star);
    let slash = just('/').to(Token::Slash);
    let l_paren = just('(').to(Token::LParen);
    let r_paren = just(')').to(Token::RParen);
    let error = any()
        .validate(|c, span, emit| {
            emit(Simple::expected_input_found(span, [], Some(c)));
            c
        })
        .map(Token::Error);

    choice((
        with, ident, number, comma, colon, plus, minus, star, slash, l_paren, r_paren,
    ))
    .or(error)
    .map_with_span(|t, span| (t, span))
    .padded()
    .repeated()
    .then_ignore(end())
}

#[test]
fn test_lexer() {
    assert_eq!(
        lexer().parse("1 2 3").unwrap(),
        vec![
            (Token::Number(1), 0..1),
            (Token::Number(2), 2..3),
            (Token::Number(3), 4..5),
        ]
    );
}
