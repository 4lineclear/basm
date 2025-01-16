use nom::{
    branch::alt,
    bytes::{
        complete::tag,
        streaming::{is_not, take_while_m_n},
    },
    character::{
        complete::{alpha1, alphanumeric1, anychar, char, one_of, space0, space1},
        streaming::multispace1,
    },
    combinator::{all_consuming, eof, map, map_opt, map_res, opt, recognize, value, verify},
    error::{FromExternalError, ParseError},
    multi::{fold_many0, many0, many0_count, many1, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult, Parser,
};
use nom_locate::LocatedSpan;

// TODO: move to using a hand-written parser.

#[cfg(test)]
mod test;

pub type Span<'a> = LocatedSpan<&'a str, u32>;

/// Turn a string into a span iterator over it's lines
pub fn line_spans(input: &str) -> impl Iterator<Item = Span> {
    let mut i = 0;
    input.lines().map(move |program| {
        let extra = i as u32;
        i += program.len() + 1;
        Span::new_extra(program, extra)
    })
}

/// Lexes each line
pub fn lex_lines(input: &str) -> impl Iterator<Item = IResult<Span, Line>> {
    line_spans(input).map(|s| line(s))
}

#[derive(Debug)]
pub struct Line<'a> {
    pub kind: LineKind<'a>,
    pub comment: Option<Span<'a>>,
}

impl<'a> From<LineKind<'a>> for Line<'a> {
    fn from(kind: LineKind<'a>) -> Self {
        Self {
            kind,
            comment: None,
        }
    }
}

#[derive(Debug)]
pub enum LineKind<'a> {
    Empty,
    Section(Span<'a>),
    Label(Span<'a>),
    Ins(Span<'a>, Vec<Value<'a>>),
    Var {
        name: Span<'a>,
        dir: Span<'a>,
        val: Vec<Value<'a>>,
    },
}

#[derive(Debug)]
pub enum Value<'a> {
    Hex(Span<'a>),
    Octal(Span<'a>),
    Binary(Span<'a>),
    Decimal(Span<'a>),
    Float(Span<'a>),
    Identifier(Span<'a>),
    Deref(Span<'a>),
    String(String),
}
pub fn line(input: Span) -> IResult<Span, Line> {
    all_consuming(map(
        pair(
            alt((
                map(eof, |_| LineKind::Empty),
                map(section, |s| LineKind::Section(s)),
                map(label, |s| LineKind::Label(s)),
                ins_or_var,
                map(space0, |_| LineKind::Empty),
            )),
            comment,
        ),
        |(kind, comment)| Line { kind, comment },
    ))(input)
}
pub fn comment(input: Span) -> IResult<Span, Option<Span>> {
    opt(spaced(recognize(pair(tag(";"), many0(anychar)))))(input)
}
pub fn section(input: Span) -> IResult<Span, Span> {
    spaced(preceded(
        tuple((tag("section"), space0_then(tag(".")))),
        space0_then(identifier),
    ))(input)
}

pub fn label(input: Span) -> IResult<Span, Span> {
    spaced(terminated(identifier, preceded(space0, tag(":"))))(input)
}

pub fn values0(input: Span) -> IResult<Span, Vec<Value>> {
    spaced(separated_list0(tag(","), spaced(val)))(input)
}

pub fn values1(input: Span) -> IResult<Span, Vec<Value>> {
    spaced(separated_list1(tag(","), spaced(val)))(input)
}

/// instruction or variable
pub fn ins_or_var(input: Span) -> IResult<Span, LineKind> {
    map(
        spaced(tuple((
            identifier,
            alt((
                map(tuple((space1_then(identifier), values1)), Ok),
                map(values0, Err),
            )),
        ))),
        |(name, args)| match args {
            Ok((dir, val)) => LineKind::Var { name, dir, val },
            Err(args) => LineKind::Ins(name, args),
        },
    )(input)
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
fn spaced<'a, F: 'a, O, E: ParseError<Span<'a>>>(
    inner: F,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O, E>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O, E>,
{
    delimited(space0, inner, space0)
}

pub fn val(input: Span) -> IResult<Span, Value> {
    alt((
        map(hexadecimal, Value::Hex),
        map(octal, Value::Octal),
        map(binary, Value::Binary),
        map(decimal, Value::Decimal),
        map(float, Value::Float),
        map(identifier, Value::Identifier),
        map(derefed, Value::Deref),
        map(parse_string, Value::String),
    ))(input)
}

pub fn derefed(input: Span) -> IResult<Span, Span> {
    delimited(tag("["), spaced(identifier), tag("]"))(input)
}

fn space0_then<'a, F: 'a, O, E: ParseError<Span<'a>>>(
    inner: F,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O, E>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O, E>,
{
    preceded(space0, inner)
}

fn space1_then<'a, F: 'a, O, E: ParseError<Span<'a>>>(
    inner: F,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O, E>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O, E>,
{
    preceded(space1, inner)
}

pub fn identifier(input: Span) -> IResult<Span, Span> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

pub fn ident2(input: Span) -> IResult<Span, Span> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

fn hexadecimal(input: Span) -> IResult<Span, Span> {
    preceded(
        alt((tag("0x"), tag("0X"))),
        recognize(many1(terminated(
            one_of("0123456789abcdefABCDEF"),
            many0(char('_')),
        ))),
    )(input)
}

fn octal(input: Span) -> IResult<Span, Span> {
    preceded(
        alt((tag("0o"), tag("0O"))),
        recognize(many1(terminated(one_of("01234567"), many0(char('_'))))),
    )(input)
}

fn binary(input: Span) -> IResult<Span, Span> {
    preceded(
        alt((tag("0b"), tag("0B"))),
        recognize(many1(terminated(one_of("01"), many0(char('_'))))),
    )(input)
}

fn decimal(input: Span) -> IResult<Span, Span> {
    recognize(many1(terminated(one_of("0123456789"), many0(char('_')))))(input)
}

fn float(input: Span) -> IResult<Span, Span> {
    alt((
        // Case one: .42
        recognize(tuple((
            char('.'),
            decimal,
            opt(tuple((one_of("eE"), opt(one_of("+-")), decimal))),
        ))), // Case two: 42e42 and 42.42e42
        recognize(tuple((
            decimal,
            opt(preceded(char('.'), decimal)),
            one_of("eE"),
            opt(one_of("+-")),
            decimal,
        ))), // Case three: 42. and 42.42
        recognize(tuple((decimal, char('.'), opt(decimal)))),
    ))(input)
}

// parser combinators are constructed from the bottom up:
// first we write parsers for the smallest elements (escaped characters),
// then combine them into larger parsers.

/// Parse a unicode sequence, of the form u{XXXX}, where XXXX is 1 to 6
/// hexadecimal numerals. We will combine this later with parse_escaped_char
/// to parse sequences like \u{00AC}.
fn parse_unicode<'a, E>(input: Span<'a>) -> IResult<Span<'a>, char, E>
where
    E: ParseError<Span<'a>> + FromExternalError<Span<'a>, std::num::ParseIntError>,
{
    // `take_while_m_n` parses between `m` and `n` bytes (inclusive) that match
    // a predicate. `parse_hex` here parses between 1 and 6 hexadecimal numerals.
    let parse_hex = take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit());

    // `preceded` takes a prefix parser, and if it succeeds, returns the result
    // of the body parser. In this case, it parses u{XXXX}.
    let parse_delimited_hex = preceded(
        char('u'),
        // `delimited` is like `preceded`, but it parses both a prefix and a suffix.
        // It returns the result of the middle parser. In this case, it parses
        // {XXXX}, where XXXX is 1 to 6 hex numerals, and returns XXXX
        delimited(char('{'), parse_hex, char('}')),
    );

    // `map_res` takes the result of a parser and applies a function that returns
    // a Result. In this case we take the hex bytes from parse_hex and attempt to
    // convert them to a u32.
    let parse_u32 = map_res(parse_delimited_hex, move |hex: Span| {
        u32::from_str_radix(hex.fragment(), 16)
    });

    // map_opt is like map_res, but it takes an Option instead of a Result. If
    // the function returns None, map_opt returns an error. In this case, because
    // not all u32 values are valid unicode code points, we have to fallibly
    // convert to char with from_u32.
    map_opt(parse_u32, std::char::from_u32).parse(input)
}

/// Parse an escaped character: \n, \t, \r, \u{00AC}, etc.
fn parse_escaped_char<'a, E>(input: Span<'a>) -> IResult<Span<'a>, char, E>
where
    E: ParseError<Span<'a>> + FromExternalError<Span<'a>, std::num::ParseIntError>,
{
    preceded(
        char('\\'),
        // `alt` tries each parser in sequence, returning the result of
        // the first successful match
        alt((
            parse_unicode,
            // The `value` parser returns a fixed value (the first argument) if its
            // parser (the second argument) succeeds. In these cases, it looks for
            // the marker characters (n, r, t, etc) and returns the matching
            // character (\n, \r, \t, etc).
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
            value('\u{08}', char('b')),
            value('\u{0C}', char('f')),
            value('\\', char('\\')),
            value('/', char('/')),
            value('"', char('"')),
        )),
    )
    .parse(input)
}

/// Parse a backslash, followed by any amount of whitespace. This is used later
/// to discard any escaped whitespace.
fn parse_escaped_whitespace<'a, E: ParseError<Span<'a>>>(
    input: Span<'a>,
) -> IResult<Span<'a>, Span<'a>, E> {
    preceded(char('\\'), multispace1).parse(input)
}

/// Parse a non-empty block of text that doesn't include \ or "
fn parse_literal<'a, E: ParseError<Span<'a>>>(input: Span<'a>) -> IResult<Span<'a>, Span<'a>, E> {
    // `is_not` parses a string of 0 or more characters that aren't one of the
    // given characters.
    let not_quote_slash = is_not("\"\\");

    // `verify` runs a parser, then runs a verification function on the output of
    // the parser. The verification function accepts out output only if it
    // returns true. In this case, we want to ensure that the output of is_not
    // is non-empty.
    verify(not_quote_slash, |s: &Span| !s.fragment().is_empty()).parse(input)
}

/// A string fragment contains a fragment of a string being parsed: either
/// a non-empty Literal (a series of non-escaped characters), a single
/// parsed escaped character, or a block of escaped whitespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(Span<'a>),
    EscapedChar(char),
    EscapedWS,
}

/// Combine parse_literal, parse_escaped_whitespace, and parse_escaped_char
/// into a StringFragment.
fn parse_fragment<'a, E>(input: Span<'a>) -> IResult<Span<'a>, StringFragment<'a>, E>
where
    E: ParseError<Span<'a>> + FromExternalError<Span<'a>, std::num::ParseIntError>,
{
    alt((
        // The `map` combinator runs a parser, then applies a function to the output
        // of that parser.
        map(parse_literal, StringFragment::Literal),
        map(parse_escaped_char, StringFragment::EscapedChar),
        value(StringFragment::EscapedWS, parse_escaped_whitespace),
    ))
    .parse(input)
}

/// Parse a string. Use a loop of parse_fragment and push all of the fragments
/// into an output string.
pub fn parse_string<'a, E>(input: Span<'a>) -> IResult<Span<'a>, String, E>
where
    E: ParseError<Span<'a>> + FromExternalError<Span<'a>, std::num::ParseIntError>,
{
    // fold is the equivalent of iterator::fold. It runs a parser in a loop,
    // and for each output value, calls a folding function on each output value.
    let build_string = fold_many0(
        // Our parser function – parses a single string fragment
        parse_fragment,
        // Our init value, an empty string
        String::new,
        // Our folding function. For each fragment, append the fragment to the
        // string.
        |mut string, fragment| {
            match fragment {
                StringFragment::Literal(s) => string.push_str(s.fragment()),
                StringFragment::EscapedChar(c) => string.push(c),
                StringFragment::EscapedWS => {}
            }
            string
        },
    );

    // Finally, parse the string. Note that, if `build_string` could accept a raw
    // " character, the closing delimiter " would never match. When using
    // `delimited` with a looping parser (like fold), be sure that the
    // loop won't accidentally match your closing delimiter!
    delimited(char('"'), build_string, char('"'))(input)
}
