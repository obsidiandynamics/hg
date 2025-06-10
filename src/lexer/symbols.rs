use std::borrow::Cow;
use std::fmt::{Display, Formatter};

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
const fn is_symbol(byte: u8) -> bool {
    SYMBOL_MAP[byte as usize]
}

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub struct SymbolString(Cow<'static, [u8]>);

impl Display for SymbolString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::from("[");
        for (index, byte) in self.0.iter().enumerate() {
            buf.push_str(format!("{:#x}", byte).as_str());
            if index < self.0.len() - 1 {
                buf.push_str(", ")
            }
        }
        write!(f, "{buf}]")
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("invalid symbol {0:#x} at offset {1}")]
pub struct InvalidSymbol(u8, usize);

impl TryFrom<&'static str> for SymbolString {
    type Error = InvalidSymbol;

    fn try_from(str: &'static str) -> Result<Self, Self::Error> {
        match str.bytes().enumerate().find(|(_, byte)| !is_symbol(*byte)) {
            None => Ok(SymbolString(str.as_bytes().into())),
            Some((index, invalid_byte)) => Err(InvalidSymbol(invalid_byte, index))
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("duplicate {0}")]
    Duplicate(SymbolString)
}

#[derive(Debug)]
pub struct SymbolTable(Cow<'static, [SymbolString]>);

impl SymbolTable {
    pub fn empty() -> Self {
        SymbolTable(Cow::default())
    }

    pub fn add(&mut self, symbol: SymbolString) -> Result<(), Error> {
        match self.0.binary_search(&symbol) {
            Ok(_) => {
                Err(Error::Duplicate(symbol))
            }
            Err(index) => {
                self.0.to_mut().insert(index, symbol);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use crate::lexer::symbols::{SymbolString, SymbolTable, SYMBOL_MAP};

    #[test]
    fn symbol_parse_valid() {
        SymbolString::try_from(":@#").unwrap();
    }
    
    #[test]
    fn symbol_parse_invalid() {
        let err = SymbolString::try_from(":@a#").unwrap_err();
        assert_eq!("invalid symbol 0x61 at offset 2", err.to_string());
    }

    #[test]
    fn symbols_add_new() -> Result<(), Box<dyn Error>> {
        let mut symbols = SymbolTable::empty();
        symbols.add(SymbolString::try_from(":")?)?;
        symbols.add(SymbolString::try_from("::")?)?;
        symbols.add(SymbolString::try_from("@")?)?;
        symbols.add(SymbolString::try_from("@#")?)?;
        symbols.add(SymbolString::try_from("@?")?)?;
        println!("symbols: {symbols:?}");
        Ok(())
    }

    #[test]
    fn symbols_add_duplicate() {
        let mut symbols = SymbolTable::empty();
        symbols.add(SymbolString::try_from(":").unwrap()).unwrap();
        symbols.add(SymbolString::try_from(":?").unwrap()).unwrap();
        let err = symbols.add(SymbolString::try_from(":?").unwrap()).unwrap_err();
        assert_eq!("duplicate [0x3a, 0x3f]", err.to_string());
    }

    const EXPECTED_SYMBOLS: &str = "!#$%&*+,-./:;<=>?@^`|~";

    #[test]
    fn all_symbols_in_table() {
        for byte in EXPECTED_SYMBOLS.bytes() {
            assert!(SYMBOL_MAP[byte as usize], "for byte {byte:#x}");
        }
    }

    #[test]
    fn no_extraneous_symbols_in_table() {
        let expected_symbol_bytes = EXPECTED_SYMBOLS.as_bytes();
        for (index, &symbol) in SYMBOL_MAP.iter().enumerate() {
            if symbol {
                assert!(expected_symbol_bytes.contains(&(index as u8)), "for index {index:#x}");
            }
        }
    }
}