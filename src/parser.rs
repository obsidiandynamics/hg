use std::collections::VecDeque;
use std::mem;
use thiserror::Error;
use crate::token::Token;
use crate::tree::Node;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unterminated container")]
    UnterminatedContainer,
    
    #[error("unterminated list")]
    UnterminatedList,

    #[error("unexpected token {0:?}")]
    UnexpectedToken(Token),

    #[error("empty list segment")]
    EmptyListSegment,
}

pub fn parse(mut tokens: VecDeque<Token>) -> Result<Vec<Node>, Error> {
    let mut nodes = vec![];
    loop {
        if let Some(token) = tokens.pop_front() {
            match token {
                Token::Text(_) | Token::Character(_) | Token::Integer(_) | Token::Decimal(_, _, _) | Token::Boolean(_) | Token::Dash | Token::Ident(_) | Token::Newline => {
                    nodes.push(Node::Raw(token))
                }
                Token::LeftParen => {
                    let child = parse_list(&mut tokens)?;
                    nodes.push(child);
                }
                Token::LeftBrace => {
                    let child = parse_container(&mut tokens)?;
                    nodes.push(child);
                }
                Token::Colon => {
                    //TODO switch to pair
                }
                Token::Comma | Token::RightParen | Token::RightBrace => {
                    return Err(Error::UnexpectedToken(token))
                }
            }
        } else {
            break;
        }
    }
    Ok(nodes)
}

fn parse_container(tokens: &mut VecDeque<Token>) -> Result<Node, Error> {
    let mut children = vec![];
    loop {
        if let Some(token) = tokens.pop_front() {
            match token {
                Token::Text(_) | Token::Character(_) | Token::Integer(_) | Token::Decimal(_, _, _) | Token::Boolean(_) | Token::Dash | Token::Ident(_) | Token::Newline => {
                    children.push(Node::Raw(token))
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
                    //TODO
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
                    segment.push(Node::Raw(token))
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
                    let new_segment = vec![];
                    let segment = mem::replace(&mut segment, new_segment);
                    segments.push(segment);
                }
                Token::Colon => {
                    //TODO
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

#[cfg(test)]
mod tests;