use std::mem;
use thiserror::Error;
use crate::lexer;
use crate::lexer::Fragment;
use crate::metadata::{Location, Metadata};
use crate::parser::fragment_stream::{FragmentStream};
use crate::token::{Ascii, ListDelimiter, Token};
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

    #[error("unterminated relation")]
    UnterminatedRelation,

    #[error("unterminated prefix")]
    UnterminatedPrefix,

    #[error("unterminated phrase")]
    UnterminatedPhrase,

    #[error("unexpected token {0:?}")]
    UnexpectedToken(Token<'a>),

    #[error("empty verse")]
    EmptyVerse,

    #[error("empty relation segment")]
    EmptyRelationSegment,
}

#[inline]
pub fn parse<'a, I: IntoIterator<Item=Fragment<'a>>>(into_iter: I) -> Result<Verse<'a>, Error<'a>> {
    let mut fragments = FragmentStream::from(into_iter.into_iter());
    let mut verse = vec![];
    let mut phrase = vec![];
    while let Some(fragment) = fragments.next() {
        let (token, metadata) = fragment?;
        match token {
            Token::Newline => {
                if !phrase.is_empty() {
                    let phrase = mem::take(&mut phrase);
                    verse.push(Phrase(phrase));
                }
            }
            Token::Left(delimiter) => {
                let child = parse_list(metadata.start, delimiter, &mut fragments)?;
                phrase.push(child);
            }
            Token::Symbol(Ascii(b':')) => {
                let head = relation_head(&mut phrase)?;
                let child = parse_relation(head, metadata.end, &mut fragments)?;
                phrase.push(child);
            }
            Token::Symbol(Ascii(b',')) | Token::Right(_) => {
                return Err(Error::UnexpectedToken(token))
            },
            Token::Text(_) | Token::Character(_) | Token::Integer(_) | Token::Decimal(_) | Token::Boolean(_) | Token::Ident(_) | Token::Symbol(_) | Token::ExtendedSymbol(_) => {
                phrase.push(Node::Raw(token, metadata));
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
fn parse_list<'a, I: Iterator<Item=Fragment<'a>>>(start: Option<Location>, left_delimiter: ListDelimiter, fragments: &mut FragmentStream<'a, I>) -> Result<Node<'a>, Error<'a>> {
    let mut verses = vec![];
    let mut verse = vec![];
    let mut phrase = vec![];
    loop {
        if let Some(fragment) = fragments.next() {
            let (token, metadata) = fragment?;
            match token {
                Token::Newline => {
                    if !phrase.is_empty() {
                        let phrase = mem::take(&mut phrase);
                        verse.push(Phrase(phrase));
                    }
                }
                Token::Left(delimiter) => {
                    let child = parse_list(metadata.start, delimiter, fragments)?;
                    phrase.push(child);
                }
                Token::Symbol(Ascii(b',')) => {
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
                Token::Symbol(Ascii(b':')) => {
                    let head = relation_head(&mut phrase)?;
                    let child = parse_relation(head, metadata.end, fragments)?;
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
                        let end = metadata.end;
                        Ok(Node::List(verses, Metadata { start, end}))
                    } else {
                        Err(Error::UnexpectedToken(Token::Right(right_delimiter)))
                    }
                },
                Token::Text(_) | Token::Character(_) | Token::Integer(_) | Token::Decimal(_) | Token::Boolean(_) | Token::Ident(_) | Token::Symbol(_) | Token::ExtendedSymbol(_)=> {
                    phrase.push(Node::Raw(token, metadata));
                }
            }
        } else {
            return Err(Error::UnterminatedList)
        }
    }
}

#[inline]
fn relation_head<'a>(nodes: &mut Vec<Node<'a>>) -> Result<Node<'a>, Error<'a>> {
    if !nodes.is_empty() {
        Ok(nodes.remove(nodes.len() - 1))
    } else {
        Err(Error::EmptyRelationSegment)
    }
}

#[inline]
fn parse_relation<'a, I: Iterator<Item=Fragment<'a>>>(head: Node<'a>, colon_location: Option<Location>, fragments: &mut FragmentStream<'a, I>) -> Result<Node<'a>, Error<'a>> {
    let mut tail = vec![];
    loop {
        if let Some(fragment) = fragments.next() {
            let (token, metadata) = fragment?;
            match token {
                Token::Left(delimiter) => {
                    let child = parse_list(metadata.start, delimiter, fragments)?;
                    tail.push(child);
                }
                Token::Right(_) | Token::Symbol(Ascii(b',')) | Token::Newline => {
                    fragments.stash(Ok((token, metadata))); // restore token for the parent parser
                    let start = head.metadata().start.clone();
                    let end = if tail.is_empty() {
                        colon_location
                    } else {
                        tail[tail.len() - 1].metadata().end.clone()
                    };
                    return Ok(Node::Relation(Box::new(head), Phrase(tail), Metadata { start, end }))
                }
                Token::Symbol(Ascii(b':')) => {
                    return if !tail.is_empty() {
                        let previous_end = tail[tail.len() - 1].metadata().end.clone();
                        let head_start = head.metadata().start.clone();
                        let wrapped = Node::Relation(Box::new(head), Phrase(tail), Metadata { start: head_start, end: previous_end });
                        let wrapper = parse_relation(wrapped, metadata.end, fragments)?;
                        Ok(wrapper)
                    } else {
                        Err(Error::EmptyRelationSegment)
                    }
                },
                Token::Text(_) | Token::Character(_) | Token::Integer(_) | Token::Decimal(_) | Token::Boolean(_) | Token::Ident(_) | Token::Symbol(_) | Token::ExtendedSymbol(_) => {
                    tail.push(Node::Raw(token, metadata))
                }
            }
        } else {
            return Err(Error::UnterminatedRelation)
        }
    }
}

#[cfg(test)]
mod tests;