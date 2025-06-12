use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use once_cell::sync::Lazy;

pub const SYMBOL_MAP: [bool; 256] = [
    /*
    0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F */
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 0
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 1
    F, T, F, T, T, T, T, F, F, F, T, T, T, T, T, T, // 2
    F, F, F, F, F, F, F, F, F, F, T, T, T, T, T, T, // 3
    T, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 4
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, T, F, // 5
    T, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 6
    F, F, F, F, F, F, F, F, F, F, F, F, T, F, T, F, // 7
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 8
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 9
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // A
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // B
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // C
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // D
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // E
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // F
];

const T: bool = true;
const F: bool = false;

#[inline(always)]
pub const fn is_symbol(byte: u8) -> bool {
    SYMBOL_MAP[byte as usize]
}

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub struct SymbolString<'a>(pub Cow<'a, [u8]>);

impl Display for SymbolString<'_> {
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

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    #[error("invalid symbol {0:#x} at offset {1}")]
    InvalidSymbol(u8, usize),

    #[error("symbol string should be at least 2 bytes long")]
    TooShort
}

impl TryFrom<&'static str> for SymbolString<'_> {
    type Error = ParseError;

    fn try_from(str: &'static str) -> Result<Self, Self::Error> {
        if str.len() >= 2 {
            match str.bytes().enumerate().find(|(_, byte)| !is_symbol(*byte)) {
                None => Ok(SymbolString(str.as_bytes().into())),
                Some((index, invalid_byte)) => Err(ParseError::InvalidSymbol(invalid_byte, index))
            }
        } else {
            Err(ParseError::TooShort)
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error<'a> {
    #[error("duplicate {0}")]
    Duplicate(SymbolString<'a>),

    #[error("missing prefix for {0}")]
    MissingPrefix(SymbolString<'a>)
}

#[derive(Debug, Clone)]
pub struct SymbolTable<'a>(Cow<'a, [SymbolString<'a>]>);

impl<'a> SymbolTable<'a> {
    pub fn empty() -> Self {
        SymbolTable(Cow::default())
    }

    pub fn contains(&self, symbol: &SymbolString) -> bool {
        self.0.binary_search(symbol).is_ok()
    }

    pub fn add(&mut self, symbol: SymbolString<'a>) -> Result<(), Error> {
        let prefix_exists = match &symbol.0 {
            Cow::Borrowed(slice) => {
                if slice.len() == 2 {
                    true
                } else {
                    let prefix = &slice[..slice.len() - 1];
                    self.contains(&SymbolString(prefix.into()))
                }
            }
            Cow::Owned(vec) => {
                if vec.len() == 2 {
                    true
                } else {
                    let prefix = &vec[..vec.len() - 1];
                    self.contains(&SymbolString(prefix.into()))
                }
            }
        };
        if prefix_exists {
            match self.0.binary_search(&symbol) {
                Ok(_) => {
                    Err(Error::Duplicate(symbol))
                }
                Err(index) => {
                    self.0.to_mut().insert(index, symbol);
                    Ok(())
                }
            }
        } else {
            Err(Error::MissingPrefix(symbol))
        }
    }
}

static DEFAULT_SYMBOL_TABLE: Lazy<SymbolTable> = Lazy::new(|| {
    let mut symbols = SymbolTable::empty();
    symbols.add(SymbolString::try_from("::").unwrap()).unwrap();
    symbols.add(SymbolString::try_from("--").unwrap()).unwrap();
    symbols.add(SymbolString::try_from("-=").unwrap()).unwrap();
    symbols.add(SymbolString::try_from("++").unwrap()).unwrap();
    symbols.add(SymbolString::try_from("+=").unwrap()).unwrap();
    symbols
});

impl Default for SymbolTable<'static> {
    #[inline]
    fn default() -> Self {
        DEFAULT_SYMBOL_TABLE.clone()
    }
}

#[cfg(test)]
mod tests;