use std::collections::VecDeque;
use std::mem;
use thiserror::Error;
use crate::token::Token;
use crate::tree::{Node, Sentence, Verse};

#[derive(Debug, Error)]
pub enum Error {
    #[error("unterminated container")]
    UnterminatedContainer,
    
    #[error("unterminated list")]
    UnterminatedList,

    #[error("unterminated cons")]
    UnterminatedCons,

    #[error("unterminated prefix")]
    UnterminatedPrefix,

    #[error("unterminated sentence")]
    UnterminatedSentence,

    #[error("unexpected token {0:?}")]
    UnexpectedToken(Token),

    #[error("empty list segment")]
    EmptyListSegment,

    #[error("empty cons segment")]
    EmptyConsSegment,
}

pub fn parse(mut tokens: VecDeque<Token>) -> Result<Verse, Error> {
    let mut verse = vec![];
    let mut sentence = vec![];
    while let Some(token) = tokens.pop_front() {
        match token {
            Token::Newline => {
                if !sentence.is_empty() {
                    let sentence = mem::take(&mut sentence);
                    verse.push(Sentence(sentence));
                }
            }
            Token::Text(_) | Token::Character(_) | Token::Integer(_) | Token::Decimal(_, _, _) | Token::Boolean(_) | Token::Ident(_) => {
                sentence.push(Node::Raw(token));
            }
            Token::LeftParen => {
                let child = parse_list(&mut tokens)?;
                sentence.push(child);
            }
            Token::LeftBrace => {
                let child = parse_container(&mut tokens)?;
                sentence.push(child);
            }
            Token::Colon => {
                let head = cons_head(&mut sentence)?;
                let child = parse_cons(head, &mut tokens)?;
                sentence.push(child);
            }
            Token::Dash => {
                let child = parse_prefix(token, &mut tokens)?;
                sentence.push(child);
            }
            Token::Comma | Token::RightParen | Token::RightBrace => {
                return Err(Error::UnexpectedToken(token))
            }
        }
    }
    
    if sentence.is_empty() {
        Ok(Verse(verse))
    } else {
        Err(Error::UnterminatedSentence)
    } 
}

fn parse_container(tokens: &mut VecDeque<Token>) -> Result<Node, Error> {
    let mut children = vec![];
    loop {
        if let Some(token) = tokens.pop_front() {
            match token {
                Token::Text(_) | Token::Character(_) | Token::Integer(_) | Token::Decimal(_, _, _) | Token::Boolean(_) | Token::Dash | Token::Ident(_) | Token::Newline => {
                    children.push(Node::Raw(token));
                }
                Token::LeftParen => {
                    let child = parse_list(tokens)?;
                    children.push(child);
                }
                Token::LeftBrace => {
                    let child = parse_container(tokens)?;
                    children.push(child);
                }
                Token::RightBrace => {
                    return Ok(Node::Container(children))
                }
                Token::Colon => {
                    let head = cons_head(&mut children)?;
                    let child = parse_cons(head, tokens)?;
                    children.push(child);
                }
                Token::Comma | Token::RightParen => {
                    return Err(Error::UnexpectedToken(token))
                }
            }
        } else {
            return Err(Error::UnterminatedContainer)
        }
    }
}

fn parse_list(tokens: &mut VecDeque<Token>) -> Result<Node, Error> {
    let mut segments = vec![];
    let mut segment = vec![];
    loop {
        if let Some(token) = tokens.pop_front() {
            match token {
                Token::Text(_) | Token::Character(_) | Token::Integer(_) | Token::Decimal(_, _, _) | Token::Boolean(_) | Token::Dash | Token::Ident(_) | Token::Newline => {
                    segment.push(Node::Raw(token));
                }
                Token::LeftParen => {
                    let child = parse_list(tokens)?;
                    segment.push(child);
                }
                Token::LeftBrace => {
                    let child = parse_container(tokens)?;
                    segment.push(child);
                }
                Token::RightBrace => {
                    return Err(Error::UnexpectedToken(token))
                }
                Token::Comma => {
                    if segment.is_empty() {
                        return Err(Error::EmptyListSegment)
                    }
                    let segment = mem::take(&mut segment);
                    segments.push(segment);
                }
                Token::Colon => {
                    let head = cons_head(&mut segment)?;
                    let child = parse_cons(head, tokens)?;
                    segment.push(child);
                }
                Token::RightParen => {
                    if !segment.is_empty() {
                        // the last segment may be empty
                        segments.push(segment);
                    }
                    return Ok(Node::List(segments))
                }
            }
        } else {
            return Err(Error::UnterminatedList)
        }
    }
}

fn cons_head(nodes: &mut Vec<Node>) -> Result<Node, Error> {
    if !nodes.is_empty() {
        Ok(nodes.remove(nodes.len() - 1))
    } else {
        Err(Error::EmptyConsSegment)
    }
}

fn parse_cons(head: Node, tokens: &mut VecDeque<Token>) -> Result<Node, Error> {
    let mut tail = vec![];
    loop {
        if let Some(token) = tokens.pop_front() {
            match token {
                Token::Text(_) | Token::Character(_) | Token::Integer(_) | Token::Decimal(_, _, _) | Token::Boolean(_) | Token::Dash | Token::Ident(_) => {
                    tail.push(Node::Raw(token))
                }
                Token::LeftParen => {
                    let child = parse_list(tokens)?;
                    tail.push(child);
                }
                Token::LeftBrace => {
                    let child = parse_container(tokens)?;
                    tail.push(child);
                }
                Token::RightBrace | Token::RightParen | Token::Comma | Token::Newline => {
                    tokens.push_front(token); // restore token for the parent parser
                    return Ok(Node::Cons(Box::new(head), Sentence(tail)))
                }
                Token::Colon => {
                    return if !tail.is_empty() {
                        let cons = Node::Cons(Box::new(head), Sentence(tail));
                        let child = parse_cons(cons, tokens)?;
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

fn parse_prefix(symbol: Token, tokens: &mut VecDeque<Token>) -> Result<Node, Error> {
    match tokens.pop_front() {
        Some(token) => {
            match token {
                Token::Text(_) | Token::Character(_) | Token::Integer(_) | Token::Decimal(_, _, _) | Token::Boolean(_) | Token::Ident(_) => {
                    Ok(Node::Prefix(symbol, Box::new(Node::Raw(token))))
                }
                Token::LeftParen => {
                    let child = parse_list(tokens)?;
                    Ok(Node::Prefix(symbol, Box::new(child)))
                }
                Token::LeftBrace => {
                    let child = parse_container(tokens)?;
                    Ok(Node::Prefix(symbol, Box::new(child)))
                }
                Token::Newline | Token::RightBrace | Token::RightParen | Token::Comma | Token::Colon | Token::Dash => {
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