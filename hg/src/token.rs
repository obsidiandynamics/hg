use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use crate::types::unqualified_type_name;

#[derive(PartialEq, Eq, Clone)]
pub struct Ascii(pub u8);

impl Debug for Ascii {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(b'{}')", unqualified_type_name::<Self>(), self.0 as char)
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct AsciiSlice<'a>(pub &'a [u8]);

impl Debug for AsciiSlice<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::from("[");
        for (index, byte) in self.0.iter().enumerate() {
            buf.push_str(format!("b'{}'", *byte as char).as_str());
            if index < self.0.len() - 1 {
                buf.push_str(", ")
            }
        }
        write!(f, "{buf}]")
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token<'a> {
    Text(Cow<'a, str>),
    Character(char),
    Integer(u128),
    Decimal(Decimal),
    Boolean(bool),
    Left(ListDelimiter),
    Right(ListDelimiter),
    Symbol(Ascii),
    ExtendedSymbol(AsciiSlice<'a>),
    Ident(Cow<'a, str>),
    Newline,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ListDelimiter {
    Paren,
    Brace,
    Bracket,
    Angle
}

/// A decimal in the form (whole part, fractional part, scale).
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Decimal(pub u128, pub u128, pub u8);

impl From<Decimal> for f64 {
    fn from(decimal: Decimal) -> Self {
        let Decimal(whole, fractional, scale) = decimal;
        whole as f64 + fractional as f64 / 10_f64.powi(scale as i32)
    }
}

#[cfg(test)]
mod tests {
    use crate::token::{Ascii, Decimal};

    #[test]
    fn byte_debug() {
        let byte = Ascii(b'a');
        assert_eq!("Ascii(b'a')", format!("{byte:?}"));
    }

    #[test]
    fn f64_from_decimal() {
        assert_eq!(7.0, f64::from(Decimal(7, 0, 1)));
        assert_eq!(7.0, f64::from(Decimal(7, 0, 2)));
        assert_eq!(7.1, f64::from(Decimal(7, 1, 1)));
        assert_eq!(7.01, f64::from(Decimal(7, 1, 2)));
        assert_eq!(7.001, f64::from(Decimal(7, 1, 3)));
        assert_eq!(7.1, f64::from(Decimal(7, 10, 2)));
        assert_eq!(7.01, f64::from(Decimal(7, 10, 3)));
        assert_eq!(7.012, f64::from(Decimal(7, 12, 3)));
        assert_eq!(7.123, f64::from(Decimal(7, 123, 3)));
    }
}