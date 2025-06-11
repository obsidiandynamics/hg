use crate::symbols::{is_symbol, SymbolString, SymbolTable, SYMBOL_MAP};

#[test]
fn symbol_parse_valid() {
    SymbolString::try_from(":@#").unwrap();
}

#[test]
fn symbol_parse_invalid_symbol_err() {
    let err = SymbolString::try_from(":@a#").unwrap_err();
    assert_eq!("invalid symbol 0x61 at offset 2", err.to_string());
}

#[test]
fn symbol_parse_too_short_err() {
    let err = SymbolString::try_from(":").unwrap_err();
    assert_eq!("symbol string should be at least 2 bytes long", err.to_string());
}

#[test]
fn symbols_add_new() {
    let mut symbols = SymbolTable::empty();
    symbols.add(SymbolString::try_from("::").unwrap()).unwrap();
    symbols.add(SymbolString::try_from(":::").unwrap()).unwrap();
    symbols.add(SymbolString::try_from("@#").unwrap()).unwrap();
    symbols.add(SymbolString::try_from("@?").unwrap()).unwrap();
    symbols.add(SymbolString::try_from("@?$").unwrap()).unwrap();
    println!("symbols: {symbols:#?}");
}

#[test]
fn symbols_add_duplicate_err() {
    let mut symbols = SymbolTable::empty();
    symbols.add(SymbolString::try_from("::").unwrap()).unwrap();
    symbols.add(SymbolString::try_from("::?").unwrap()).unwrap();
    let err = symbols.add(SymbolString::try_from("::?").unwrap()).unwrap_err();
    assert_eq!("duplicate [b':', b':', b'?']", err.to_string());
}

#[test]
fn symbols_add_missing_prefix_err() {
    let mut symbols = SymbolTable::empty();
    symbols.add(SymbolString::try_from("::").unwrap()).unwrap();
    symbols.add(SymbolString::try_from("::?").unwrap()).unwrap();
    let err = symbols.add(SymbolString::try_from(":?%").unwrap()).unwrap_err();
    assert_eq!("missing prefix for [b':', b'?', b'%']", err.to_string());
}

const EXPECTED_SYMBOLS: &str = "!#$%&*+,-./:;<=>?@^`|~";

#[test]
fn all_symbols_in_table() {
    for byte in EXPECTED_SYMBOLS.bytes() {
        assert!(is_symbol(byte), "for byte {byte:#x}");
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