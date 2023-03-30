use std::io::prelude::*;

/// Read lines from stdin, and seperate them by empty lines.
///
/// Output: A vector of string, each representing a section of input.
#[deprecated]
pub fn read_and_split(input: &mut dyn BufRead) -> Result<Vec<String>, std::io::Error> {
    let mut out : Vec<String> = vec![];
    let mut buf = String::new();
    for line in input.lines() {
        let line = line?;
        if line.trim().is_empty() {
            out.push(buf);
            buf = String::new();
        } else {
            buf.push_str(&line);
            buf.push('\n');
        }
    };
    if !buf.is_empty() {
        out.push(buf);
    }
    Ok(out)
}

fn iter_by_sections<I: BufRead + ?Sized>(input: &mut I) -> impl Iterator<Item=std::io::Result<String>> + '_ {
    let mut lines = input.lines();
    std::iter::from_fn (move || -> Option<std::io::Result<String>>{
        let mut buf = String::new();
        #[allow(clippy::while_let_on_iterator)]
        while let Some(line) = lines.next() {
            let line = match line {
                Ok(line) => line,
                e@Err(_) => return Some(e),
            };
            if line.trim().is_empty() {
                return Some(Ok(buf));
            } else {
                if !buf.is_empty() {
                    buf.push('\n');
                }
                buf.push_str(&line);
            }
        }
        if buf.is_empty() {
            None
        } else {
            Some(Ok(buf))
        }
    })
}

/// Read lines from stdin, and seperate them by empty lines.
pub fn sections(input: &mut dyn BufRead) -> impl Iterator<Item=std::io::Result<String>> + '_ {
    iter_by_sections(input)
}

/// Read everything from stdin
pub fn read_stdin() -> std::io::Result<String> {
    let mut s = String::new();
    std::io::stdin().lock().read_to_string(&mut s)?;
    Ok(s)
}

pub fn read_stdin_by_section() -> impl Iterator<Item=std::io::Result<Vec<String>>> {
    std::iter::from_fn(|| {
        let stdin = std::io::stdin().lock();
        let lines = stdin.lines();
        let mut results = Vec::new();
        for line in lines {
            let line = match line {
                Ok(x) => x,
                Err(e) => return Some(Err(e)),
            };
            if line.is_empty() {
                break
            } else {
                results.push(line);
            }
        }
        if results.is_empty() {
            None
        } else {
            Some(Ok(results))
        }

    })


}

#[cfg(test)]
mod test {
    use indoc::indoc;
    use super::*;
    #[test]
    fn test_read_and_split() {
        const SAMPLE_IN : &str = indoc!{ "
        a
        b
        c

        d
        e"};
        let mut buf = SAMPLE_IN.as_bytes();
        assert_eq!(read_and_split(&mut buf).unwrap(), vec!["a\nb\nc\n", "d\ne\n"]);
    }
}
