#[derive(Clone, Eq, PartialEq, Debug)]
enum Signal {
    Integer(i32),
    List(Vec<Signal>),
}

impl Ord for Signal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::iter::once;
        use Signal::*;
        match (self, other) {
            (Integer(i), Integer(j)) => i.cmp(j),
            (List(v), List(u)) => v.cmp(u),
            (&Integer(i), List(u)) => once(&Signal::Integer(i)).cmp(u.iter()),
            (List(v), &Integer(j)) => v.iter().cmp(once(&Signal::Integer(j))),
        }
    }
}

impl PartialOrd for Signal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

#[allow(unused)]
mod parse {
    use super::*;
    use nom::{
        branch::alt,
        character::complete::{char, i32, multispace0, newline},
        combinator::map,
        multi::separated_list0,
        sequence::{delimited, pair, separated_pair, terminated},
        Finish, IResult,
    };

    type ParseResult<'a, T> = Result<T, nom::error::Error<String>>;

    fn integer(s: &str) -> IResult<&str, Signal> {
        map(i32, Signal::Integer)(s)
    }

    fn list(s: &str) -> IResult<&str, Signal> {
        map(
            delimited(char('['), separated_list0(char(','), signal), char(']')),
            Signal::List,
        )(s)
    }

    fn signal(s: &str) -> IResult<&str, Signal> {
        alt((integer, list))(s)
    }

    pub(super) fn parse_signal(s: &str) -> ParseResult<Signal> {
        signal(s)
            .map_err(|e| e.to_owned())
            .finish()
            .map(|(_, out)| out)
    }
    pub(super) fn parse(s: &str) -> ParseResult<Vec<(Signal, Signal)>> {
        terminated(
            separated_list0(
                pair(newline, newline),
                separated_pair(signal, newline, signal),
            ),
            multispace0,
        )(s)
        .map_err(|e| e.to_owned())
        .finish()
        .map(|(_, out)| out)
    }
}

#[allow(unused)]
mod my_parse {
    use super::Signal;

    #[derive(PartialEq, Eq, Debug, thiserror::Error)]
    pub enum ParseError {
        #[error("Unexpected end of input")]
        Eof,
        #[error("Unexpected Token: {0:?}")]
        UnexpectedToken(Token),
        #[error("Expected empty line")]
        ExpectEmptyLine,
    }

    pub type ParseResult<T> = Result<T, ParseError>;
    #[derive(Eq, PartialEq, Debug)]
    pub enum Token {
        LeftBracket,
        RightBracket,
        Integer(i32),
        Comma,
        Unrecognized(char),
    }

    type TokenStream<'a> = std::iter::Peekable<Box<dyn Iterator<Item = Token> + 'a>>;

    fn tokens(s: &str) -> TokenStream {
        let mut chars = s.chars().peekable();
        let token_stream = std::iter::from_fn(move || {
            while let Some(_) = chars.next_if(|c| c.is_whitespace()) {}
            let token = match chars.next()? {
                '[' => Token::LeftBracket,
                ']' => Token::RightBracket,
                ',' => Token::Comma,
                c if c.is_ascii_digit() => {
                    fn to_digit(c: char) -> i32 {
                        c.to_digit(10)
                            .expect("This cannot fail because we already checked ascii_digit")
                            as i32
                    }
                    let mut val = to_digit(c);
                    while let Some(c) = chars.next_if(|c| c.is_ascii_digit()) {
                        val *= 10;
                        val += to_digit(c);
                    }
                    Token::Integer(val)
                }
                c => Token::Unrecognized(c),
            };

            Some(token)
        });
        let token_stream: Box<dyn Iterator<Item = Token>> = Box::new(token_stream);
        token_stream.peekable()
    }

    fn signal<'s, 't: 's>(tok: &'s mut TokenStream<'t>) -> ParseResult<Signal> {
        fn token_items(tok: &mut TokenStream) -> ParseResult<Vec<Signal>> {
            // consume ']' or a signal,
            // then consume '[,signal]'
            if let Some(&Token::RightBracket) = tok.peek() {
                let _ = tok.next();
                return Ok(vec![]);
            }
            let mut signals = vec![];
            loop {
                signals.push(signal(tok)?);
                match tok.peek() {
                    Some(&Token::RightBracket) => {
                        let _ = tok.next();
                        return Ok(signals);
                    }
                    Some(&Token::Comma) => {
                        let _ = tok.next();
                    }
                    Some(_) => (),
                    None => return Err(ParseError::Eof),
                }
            }
        }
        match tok.next() {
            None => Err(ParseError::Eof),
            Some(Token::LeftBracket) => Ok(Signal::List(token_items(tok)?)),
            Some(Token::Integer(i)) => Ok(Signal::Integer(i)),
            Some(tok) => Err(ParseError::UnexpectedToken(tok)),
        }
    }

    pub(super) fn parse_signal(s: &str) -> ParseResult<Signal> {
        let mut token_stream = tokens(s);
        signal(&mut token_stream)
    }

    pub(super) fn parse(s: &str) -> ParseResult<Vec<(Signal, Signal)>> {
        let mut lines = s.lines();
        let mut out = vec![];
        loop {
            let fst = {
                let line = match lines.next() {
                    Some(x) => x,
                    None => return Ok(out),
                };
                if line.trim().is_empty() {
                    return Ok(out);
                }
                parse_signal(line)?
            };
            let snd = parse_signal(lines.next().ok_or(ParseError::Eof)?)?;
            let sep = match lines.next() {
                Some(x) => x,
                None => {
                    out.push((fst, snd));
                    return Ok(out);
                }
            };
            if !sep.trim().is_empty() {
                return Err(ParseError::ExpectEmptyLine);
            }
            out.push((fst, snd));
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        #[test]
        fn test_single_token() {
            assert!(tokens("[").eq([Token::LeftBracket]));
            assert!(tokens("]").eq([Token::RightBracket]));
            assert!(tokens("[]").eq([Token::LeftBracket, Token::RightBracket]));
            assert!(tokens("0").eq([Token::Integer(0)]));
            assert!(tokens("1234").eq([Token::Integer(1234)]));
            assert!(tokens(",").eq([Token::Comma]));
        }
        #[test]
        fn test_tokens_strip_whitespace() {
            assert!(tokens(" ").eq([]));
            assert!(tokens(",   ").eq([Token::Comma]));
            assert!(tokens("   ,").eq([Token::Comma]));
            assert!(tokens(" ,  ").eq([Token::Comma]));
            assert!(tokens(" ,, ").eq([Token::Comma, Token::Comma]));
            assert!(tokens(" , , ").eq([Token::Comma, Token::Comma]));
        }
        #[test]
        fn test_tokens_multiple_tokens() {
            assert!(tokens("[123, 4, 555  ] ").eq([
                Token::LeftBracket,
                Token::Integer(123),
                Token::Comma,
                Token::Integer(4),
                Token::Comma,
                Token::Integer(555),
                Token::RightBracket,
            ]))
        }

        #[test]
        fn test_signal() {
            assert_eq!(parse_signal("123"), Ok(Signal::Integer(123)));
            assert_eq!(parse_signal("[]"), Ok(Signal::List(vec![])));
            assert_eq!(
                parse_signal("[123, 456, 789]"),
                Ok(Signal::List(vec![
                    Signal::Integer(123),
                    Signal::Integer(456),
                    Signal::Integer(789)
                ]))
            );
            assert_eq!(
                parse_signal("[[[]]]"),
                Ok(Signal::List(vec![Signal::List(vec![Signal::List(vec![])])]))
            );
            assert_eq!(
                parse_signal("[123, [4], 5, []]"),
                Ok(Signal::List(vec![
                    Signal::Integer(123),
                    Signal::List(vec![Signal::Integer(4)]),
                    Signal::Integer(5),
                    Signal::List(vec![]),
                ]))
            );
        }
    }
}

fn main() -> anyhow::Result<()> {
    let input = std::io::read_to_string(std::io::stdin())?;
    let input = my_parse::parse(input.as_str())?;

    println!(
        "q1: {}",
        input
            .iter()
            .zip(1..)
            .filter_map(|((x, y), i)| (x <= y).then_some(i))
            .sum::<usize>()
    );
    let sentinels = [
        my_parse::parse_signal("[[2]]")?,
        my_parse::parse_signal("[[6]]")?,
    ];
    let mut input = input
        .into_iter()
        .flat_map(|(x, y)| [x, y])
        .chain(sentinels.clone())
        .collect::<Vec<_>>();
    input.sort();
    println!(
        "q2: {}",
        input
            .into_iter()
            .zip(1..)
            .filter_map(|(v, i)| sentinels.contains(&v).then_some(i))
            .product::<usize>()
    );
    Ok(())
}
