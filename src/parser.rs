use std::collections::VecDeque;
use thiserror::Error;
use crate::token::Token;
use crate::tree::Node;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unexpected end of stream")]
    UnexpectedEndOfStream,

    #[error("unexpected token {0:?}")]
    UnexpectedToken(Token),
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
                Token::LeftParen => {}
                Token::LeftBrace => {
                    let child = parse_container(tokens)?;
                    children.push(child);
                }
                Token::RightBrace => {
                    return Ok(Node::Container(children))
                }
                Token::Colon => {}
                Token::Comma | Token::RightParen => {
                    return Err(Error::UnexpectedToken(token))
                }
            }
        } else {
            return Err(Error::UnexpectedEndOfStream)
        }
    }
}

#[cfg(test)]
mod tests;