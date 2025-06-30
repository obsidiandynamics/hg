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
pub fn parse<'a, I: IntoIterator<Item=Fragment<'a>>>(into_iter: I) -> Result<Option<Verse<'a>>, Error<'a>> {
    let mut fragments = FragmentStream::from(into_iter.into_iter());
    let mut verse = vec![];
    let mut phrase = vec![];
    while let Some(fragment) = fragments.next() {
        let (token, metadata) = fragment?;
        match token {
            Token::Newline => {
                if !phrase.is_empty() {
                    let phrase: Vec<Node> = mem::take(&mut phrase);
                    let start = phrase[0].metadata().start.clone();
                    let end = phrase[phrase.len() - 1].metadata().end.clone();
                    verse.push(Phrase::new(phrase, Metadata { start, end }));
                }
            }
            Token::Left(delimiter) => {
                let child = parse_list(metadata.start, delimiter, &mut fragments)?;
                phrase.push(child);
            }
            Token::Symbol(Ascii(b':')) => {
                let head = relation_head(&mut phrase)?;
                let child = parse_relation(head, &mut fragments)?;
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
        if verse.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Verse::new(verse)))
        }
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
                        let phrase: Vec<Node> = mem::take(&mut phrase);
                        let start = phrase[0].metadata().start.clone();
                        let end = phrase[phrase.len() - 1].metadata().end.clone();
                        verse.push(Phrase::new(phrase, Metadata { start, end }));
                    }
                }
                Token::Left(delimiter) => {
                    let child = parse_list(metadata.start, delimiter, fragments)?;
                    phrase.push(child);
                }
                Token::Symbol(Ascii(b',')) => {
                    if !phrase.is_empty() {
                        let phrase = mem::take(&mut phrase);
                        let start = phrase[0].metadata().start.clone();
                        let end = phrase[phrase.len() - 1].metadata().end.clone();
                        verse.push(Phrase::new(phrase, Metadata { start, end }));
                    }
                    if verse.is_empty() {
                        return Err(Error::EmptyVerse)
                    }
                    let verse = mem::take(&mut verse);
                    verses.push(Verse::new(verse));
                }
                Token::Symbol(Ascii(b':')) => {
                    let head = relation_head(&mut phrase)?;
                    let child = parse_relation(head, fragments)?;
                    phrase.push(child);
                }
                Token::Right(right_delimiter) => {
                    return if left_delimiter == right_delimiter {
                        let end = metadata.end;
                        if !phrase.is_empty() {
                            let start = phrase[0].metadata().start.clone();
                            let end = phrase[phrase.len() - 1].metadata().end.clone();
                            verse.push(Phrase::new(phrase, Metadata { start, end }));
                        }
                        if !verse.is_empty() {
                            verses.push(Verse::new(verse));
                        }
                        Ok(Node::List(verses, Metadata { start, end }))
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
fn parse_relation<'a, I: Iterator<Item=Fragment<'a>>>(head: Node<'a>, fragments: &mut FragmentStream<'a, I>) -> Result<Node<'a>, Error<'a>> {
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
                    return if !tail.is_empty() {
                        let head_start = head.metadata().start.clone();
                        let tail_start = tail[0].metadata().start.clone();
                        let tail_end = tail[tail.len() - 1].metadata().end.clone();
                        let phrase = Phrase::new(tail, Metadata { start: tail_start, end: tail_end.clone() });
                        Ok(Node::Relation(Box::new(head), phrase, Metadata { start: head_start, end: tail_end }))
                    } else {
                        Err(Error::EmptyRelationSegment)
                    }
                }
                Token::Symbol(Ascii(b':')) => {
                    return if !tail.is_empty() {
                        let tail_start = tail[0].metadata().start.clone();
                        let tail_end = tail[tail.len() - 1].metadata().end.clone();
                        let head_start = head.metadata().start.clone();
                        let phrase = Phrase::new(tail, Metadata { start: tail_start, end: tail_end.clone()});
                        let wrapped = Node::Relation(Box::new(head), phrase, Metadata { start: head_start, end: tail_end });
                        let wrapper = parse_relation(wrapped, fragments)?;
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