//! Numeric parser

use exception::{Exception, RESULT_OUT_OF_RANGE};

#[derive(PartialEq, Debug)]
pub enum IResult<'l, T> {
    Done(&'l [u8], T),
    Err(Exception),
}

pub fn sign(input: &[u8]) -> IResult<isize> {
    let mut sign = 1;
    let mut bytes = input;
    if bytes.len() >= 1 {
        match bytes[0] {
            b'+' => {
                bytes = &bytes[1..];
            }
            b'-' => {
                sign = -1;
                bytes = &bytes[1..];
            }
            _ => {
                // Do nothing.
            }
        }
    }
    IResult::Done(&bytes, sign)
}

pub fn base(input: &[u8], default_base: isize) -> IResult<isize> {
    let mut base = default_base;
    let mut bytes = input;
    if bytes.len() >= 1 {
        match bytes[0] {
            b'%' => {
                base = 2;
                bytes = &bytes[1..];
            }
            b'#' => {
                base = 10;
                bytes = &bytes[1..];
            }
            b'$' => {
                base = 16;
                bytes = &bytes[1..];
            }
            _ => {
                // Do nothing.
            }
        }
    }
    IResult::Done(&bytes, base)
}

pub fn uint_in_base(input: &[u8], base: isize) -> IResult<isize> {
    let mut len = 0;
    let mut bytes = input;
    let mut value = 0isize;
    for c in bytes.iter() {
        let d;
        if b'0' <= *c && *c <= b'9' {
            d = (*c - b'0') as isize;
        } else if b'a' <= *c && *c <= b'f' {
            d = (*c - b'a') as isize + 10;
        } else if b'A' <= *c && *c <= b'F' {
            d = (*c - b'A') as isize + 10;
        } else {
            return IResult::Err(RESULT_OUT_OF_RANGE);
        }
        if d >= base {
            return IResult::Err(RESULT_OUT_OF_RANGE);
        }
        // Allow wrapping for integer.
        value = value.wrapping_mul(base).wrapping_add(d);
        len = len + 1;
    }
    bytes = &bytes[len..];
    IResult::Done(bytes, value)
}

pub fn uint(input: &[u8]) -> IResult<isize> {
    let mut len = 0;
    let mut bytes = input;
    let mut value = 0isize;
    for c in bytes.iter() {
        if b'0' <= *c && *c <= b'9' {
            // Do not allow wrapping for floating point.
            match value
                .checked_mul(10)
                .and_then(|x| x.checked_add((*c - b'0') as isize))
            {
                Some(v) => {
                    value = v;
                }
                None => return IResult::Err(RESULT_OUT_OF_RANGE),
            }
            len = len + 1;
        } else {
            break;
        }
    }
    bytes = &bytes[len..];
    IResult::Done(bytes, value)
}

/// Character in ''
pub fn quoted_char(input: &[u8]) -> IResult<isize> {
    if input.len() == 3 && input[0] == 39u8 && input[2] == 39u8 {
        IResult::Done(&input[3..], input[1] as isize)
    } else {
        IResult::Err(RESULT_OUT_OF_RANGE)
    }
}

pub fn ascii(input: &[u8], ascii: u8) -> IResult<bool> {
    if input.len() >= 1 && input[0] == ascii {
        IResult::Done(&input[1..], true)
    } else {
        IResult::Done(input, false)
    }
}

pub fn fraction(input: &[u8]) -> IResult<f64> {
    let mut len = 0i32;
    let mut bytes = input;
    let mut value = 0isize;
    if bytes.len() >= 1 && bytes[0] == b'.' {
        bytes = &bytes[1..];
        for c in bytes.iter() {
            if b'0' <= *c && *c <= b'9' {
                // Do not allow wrapping for floating point.
                match value
                    .checked_mul(10)
                    .and_then(|x| x.checked_add((*c - b'0') as isize))
                {
                    Some(v) => value = v,
                    None => return IResult::Err(RESULT_OUT_OF_RANGE),
                }
                len = len + 1;
            } else {
                break;
            }
        }
        IResult::Done(&bytes[len as usize..], (value as f64) / (10.0f64).powi(len))
    } else {
        IResult::Done(input, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign() {
        assert_eq!(sign(b"+"), IResult::Done(b"", 1));
        assert_eq!(sign(b"+x"), IResult::Done(b"x", 1));
        assert_eq!(sign(b"-"), IResult::Done(b"", -1));
        assert_eq!(sign(b"-6"), IResult::Done(b"6", -1));
        assert_eq!(sign(b""), IResult::Done(b"", 1));
        assert_eq!(sign(b"xyz"), IResult::Done(b"xyz", 1));
    }

    #[test]
    fn test_base() {
        assert_eq!(base(b"", 7), IResult::Done(b"", 7));
        assert_eq!(base(b"-", 7), IResult::Done(b"-", 7));
        assert_eq!(base(b"'", 7), IResult::Done(b"'", 7));
        assert_eq!(base(b"%", 7), IResult::Done(b"", 2));
        assert_eq!(base(b"#", 7), IResult::Done(b"", 10));
        assert_eq!(base(b"$", 7), IResult::Done(b"", 16));
    }

    #[test]
    fn test_uint() {
        assert_eq!(uint(b"123"), IResult::Done(b"", 123));
        assert_eq!(uint(b"45x6"), IResult::Done(b"x6", 45));
        assert_eq!(uint(b""), IResult::Done(b"", 0));
        assert_eq!(uint(b"xy"), IResult::Done(b"xy", 0));
    }

    #[test]
    fn test_quoted_char() {
        assert_eq!(quoted_char(b"'''"), IResult::Done(b"", 39));
        assert_eq!(quoted_char(b"'*'"), IResult::Done(b"", 42));
        assert_eq!(quoted_char(b"''"), IResult::Err(RESULT_OUT_OF_RANGE));
        assert_eq!(quoted_char(b""), IResult::Err(RESULT_OUT_OF_RANGE));
    }

    #[test]
    fn test_ascii() {
        assert_eq!(ascii(b".123", b'.'), IResult::Done(b"123", true));
        assert_eq!(ascii(b".123", b'x'), IResult::Done(b".123", false));
    }

    #[test]
    fn test_fraction() {
        assert_eq!(fraction(b"."), IResult::Done(b"", 0.0));
        assert_eq!(fraction(b".123"), IResult::Done(b"", 0.123));
        assert_eq!(fraction(b".12x"), IResult::Done(b"x", 0.12));
        assert_eq!(fraction(b"x."), IResult::Done(b"x.", 0.0));
    }
}
