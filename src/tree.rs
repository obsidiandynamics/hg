use crate::token::Token;

#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    Raw(Token),
    List(Vec<Vec<Node>>),
    Container(Vec<Node>),
    Cons(Box<Node>, Vec<Node>)
}