use std::mem;
use thiserror::Error;
use crate::lexer;
use crate::lexer::Fragment;
use crate::parser::fragment_stream::{FragmentStream};
use crate::token::{ListDelimiter, Token};
use crate::tree::{Node, Phrase, Verse};

mod fragment_stream;

#[derive(Debug, Error)]
pub enum Error<'a> {
    #[error("lexer error: {0}")]
    Lexer(#[from] Box<lexer::Error>),
    
    #[error("unterminated container")]
    UnterminatedContainer,
    
    #[error("unterminated list")]
    UnterminatedList,

    #[error("unterminated cons")]
    UnterminatedCons,

    #[error("unterminated prefix")]
    UnterminatedPrefix,

    #[error("unterminated phrase")]
    UnterminatedPhrase,

    #[error("unexpected token {0:?}")]
    UnexpectedToken(Token<'a>),

    #[error("empty verse")]
    EmptyVerse,

    #[error("empty cons segment")]
    EmptyConsSegment,
}

#[inline]
pub fn parse<'a, I: IntoIterator<Item=Fragment<'a>>>(into_iter: I) -> Result<Verse<'a>, Error<'a>> {
    let mut fragments = FragmentStream::from(into_iter.into_iter());
    let mut verse = vec![];
    let mut phrase = vec![];
    while let Some(fragment) = fragments.next() {
        let token = fragment?;
        match token {
            Token::Newline => {
                if !phrase.is_empty() {
                    let phrase = mem::take(&mut phrase);
                    verse.push(Phrase(phrase));
                }
            }
            Token::Text(_) | Token::Character(_) | Token::Integer(_) | Token::Decimal(_, _, _) | Token::Boolean(_) | Token::Ident(_) => {
                phrase.push(Node::Raw(token));
            }
            Token::Left(delimiter) => {
                let child = parse_list(delimiter, &mut fragments)?;
                phrase.push(child);
            }
            Token::Colon => {
                let head = cons_head(&mut phrase)?;
                let child = parse_cons(head, &mut fragments)?;
                phrase.push(child);
            }
            Token::Dash => {
                let child = parse_prefix(token, &mut fragments)?;
                phrase.push(child);
            }
            Token::Comma | Token::Right(_) => {
                return Err(Error::UnexpectedToken(token))
            }
        }
    }

    if phrase.is_empty() {
        Ok(Verse(verse))
    } else {
        Err(Error::UnterminatedPhrase)
    }
}

#[inline]
fn parse_list<'a, I: Iterator<Item=Fragment<'a>>>(left_delimiter: ListDelimiter, fragments: &mut FragmentStream<'a, I>) -> Result<Node<'a>, Error<'a>> {
    let mut verses = vec![];
    let mut verse = vec![];
    let mut phrase = vec![];
    loop {
        if let Some(fragment) = fragments.next() {
            let token = fragment?;
            match token {
                Token::Newline => {
                    if !phrase.is_empty() {
                        let phrase = mem::take(&mut phrase);
                        verse.push(Phrase(phrase));
                    }
                }
                Token::Text(_) | Token::Character(_) | Token::Integer(_) | Token::Decimal(_, _, _) | Token::Boolean(_) | Token::Ident(_) => {
                    phrase.push(Node::Raw(token));
                }
                Token::Left(delimiter) => {
                    let child = parse_list(delimiter, fragments)?;
                    phrase.push(child);
                }
                Token::Dash => {
                    let child = parse_prefix(token, fragments)?;
                    phrase.push(child);
                }
                Token::Comma => {
                    if !phrase.is_empty() {
                        let phrase = mem::take(&mut phrase);
                        verse.push(Phrase(phrase));
                    }
                    if verse.is_empty() {
                        return Err(Error::EmptyVerse)
                    }
                    let verse = mem::take(&mut verse);
                    verses.push(Verse(verse));
                }
                Token::Colon => {
                    let head = cons_head(&mut phrase)?;
                    let child = parse_cons(head, fragments)?;
                    phrase.push(child);
                }
                Token::Right(right_delimiter) => {
                    return if left_delimiter == right_delimiter {
                        if !phrase.is_empty() {
                            verse.push(Phrase(phrase));
                        }
                        if !verse.is_empty() {
                            verses.push(Verse(verse));
                        }
                        Ok(Node::List(verses))
                    } else {
                        Err(Error::UnexpectedToken(Token::Right(right_delimiter)))
                    }
                }
            }
        } else {
            return Err(Error::UnterminatedList)
        }
    }
}

#[inline]
fn cons_head<'a>(nodes: &mut Vec<Node<'a>>) -> Result<Node<'a>, Error<'a>> {
    if !nodes.is_empty() {
        Ok(nodes.remove(nodes.len() - 1))
    } else {
        Err(Error::EmptyConsSegment)
    }
}

#[inline]
fn parse_cons<'a, I: Iterator<Item=Fragment<'a>>>(head: Node<'a>, fragments: &mut FragmentStream<'a, I>) -> Result<Node<'a>, Error<'a>> {
    let mut tail = vec![];
    loop {
        if let Some(fragment) = fragments.next() {
            let token = fragment?;
            match token {
                Token::Text(_) | Token::Character(_) | Token::Integer(_) | Token::Decimal(_, _, _) | Token::Boolean(_) | Token::Ident(_) => {
                    tail.push(Node::Raw(token))
                }
                Token::Left(delimiter) => {
                    let child = parse_list(delimiter, fragments)?;
                    tail.push(child);
                }
                Token::Dash => {
                    let child = parse_prefix(token, fragments)?;
                    tail.push(child);
                }
                Token::Right(_) | Token::Comma | Token::Newline => {
                    fragments.stash(Ok(token)); // restore token for the parent parser
                    return Ok(Node::Cons(Box::new(head), Phrase(tail)))
                }
                Token::Colon => {
                    return if !tail.is_empty() {
                        let cons = Node::Cons(Box::new(head), Phrase(tail));
                        let child = parse_cons(cons, fragments)?;
                        Ok(child)
                    } else {
                        Err(Error::EmptyConsSegment)
                    }
                }
            }
        } else {
            return Err(Error::UnterminatedCons)
        }
    }
}

#[inline]
fn parse_prefix<'a, I: Iterator<Item=Fragment<'a>>>(symbol: Token<'a>, fragments: &mut FragmentStream<'a, I>) -> Result<Node<'a>, Error<'a>> {
    match fragments.next() {
        Some(fragment) => {
            let token = fragment?;
            match token {
                Token::Text(_) | Token::Character(_) | Token::Integer(_) | Token::Decimal(_, _, _) | Token::Boolean(_) | Token::Ident(_) => {
                    Ok(Node::Prefix(symbol, Box::new(Node::Raw(token))))
                }
                Token::Left(delimiter) => {
                    let child = parse_list(delimiter, fragments)?;
                    Ok(Node::Prefix(symbol, Box::new(child)))
                }
                Token::Newline | Token::Right(_) | Token::Comma | Token::Colon | Token::Dash => {
                    Err(Error::UnexpectedToken(token))
                }
            }
        }
        None => {
            Err(Error::UnterminatedPrefix)
        },
    }
}

#[cfg(test)]
mod tests;