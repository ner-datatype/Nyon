use combine::*;
use combine::parser::char::*;
use combine::stream::{state::{SourcePosition, State}, easy};

pub enum Token {
    Ident{s: String},
    Keyword{kw: Keyword},
    Sep{sep: Sep},
    Op{op: Op},
    Lit{lit: Lit},
}

pub enum Keyword {
    Case,
    Let,
    Type,
}

pub enum Sep {
    OpenParen,
    CloseParen,
    Dollar,
    LF,
}

pub enum Op {
    Arrow,
    Typing,
    Tuple,
    Domain,
    Hole,
    Lambda,
    UserDef{s: String},
}

pub enum Lit {
    Nat{n: ::num::BigInt},
    Int{i: ::num::BigInt},
    Str{s: String},
}

fn top_level<I>() -> impl Parser<Input = I, Output = Vec<Token>>
    where I: Stream<Item = char>,
          I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let spaces = || many::<(),_>(not_followed_by(newline()).skip(space()));
    spaces().with(many(lex().skip(spaces())))
}

fn lex<I>() -> impl Parser<Input = I, Output = Token>
    where I: Stream<Item = char>,
          I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let kw = || choice((
        string("case").map(|_| Keyword::Case),
        string("let").map(|_| Keyword::Let),
        string("type").map(|_| Keyword::Type),
    ));

    let sep = || choice((
        token('(').map(|_| Sep::OpenParen),
        token(')').map(|_| Sep::CloseParen),
        token('$').map(|_| Sep::Dollar),
        newline().or(token(';')).map(|_| Sep::LF),
    ));

    use unicode_categories::UnicodeCategories;
    let op = || choice((
        string("->").map(|_| Op::Arrow),
        token(':').map(|_| Op::Typing),
        token(',').map(|_| Op::Tuple),
        string("::").map(|_| Op::Domain),
        token('_').map(|_| Op::Hole),
        token('\\').map(|_| Op::Lambda),
        many1::<String,_>(satisfy(|c:char| c.is_punctuation_other() || c.is_symbol_math()))
            .map(|s| Op::UserDef{s}),
    ));

    choice((
        attempt( ident().map(|s| Token::Ident{s}) ),
        attempt( kw().map(|kw| Token::Keyword{kw}) ),
        attempt( sep().map(|sep| Token::Sep{sep}) ),
        attempt( op().map(|op| Token::Op{op}) ),
        attempt( lit().map(|lit| Token::Lit{lit}) ),
    ))
}

fn ident<I>() -> impl Parser<Input = I, Output = String>
    where I: Stream<Item = char>,
          I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    use std::iter::{FromIterator, once};
    let us = || token('_');

    (many::<String,_>(digit().or(us())), letter(), many::<String, _>(alpha_num().or(us())))
        .map( |(sl,ch,sr)| String::from_iter( sl.chars().chain(once(ch)).chain(sr.chars())))
}

fn lit<I>() -> impl Parser<Input = I, Output = Lit>
    where I: Stream<Item = char>,
          I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    choice((
        attempt( nat().map(|n| Lit::Nat{n}) ),
        attempt( int().map(|i| Lit::Int{i}) ),
        attempt( str().map(|s| Lit::Str{s}) ),
    ))
}

fn nat<I>() -> impl Parser<Input = I, Output = ::num::BigInt>
    where I: Stream<Item = char>,
          I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let digits = many1::<String, _>(digit());
    digits.map(|ds| ::num::Num::from_str_radix(&ds, 10).unwrap())
}

fn int<I>() -> impl Parser<Input = I, Output = ::num::BigInt>
    where I: Stream<Item = char>,
          I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let sign = optional(token('+').or(token('-')));
    sign.and(nat()).map(|(s, n): (Option<char>, ::num::BigInt)| if s == Some('-') {-n} else {n})
}

fn str<I>() -> impl Parser<Input = I, Output = String>
    where I: Stream<Item = char>,
          I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    use combine::error::Consumed;

    token('"').with(many::<String, _>( satisfy(|s| s != '"').then(|c| {
        parser(move |input|
            if c == '\\' {
                satisfy(|s| s == '\\' || s == '"' || s == 'n').map(|d| match d {
                    '\\' | '"' => d,
                    'n' => '\n',
                    _ => unreachable!()
                }).parse_stream(input)
            }
            else {
                Ok((c, Consumed::Empty(())))
            }
        )
    }) )).skip(token('"'))
}