use crate::token::Token;

#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    Raw(Token),
    List(Vec<Vec<Node>>),
    Container(Vec<Node>),
    Cons(Box<Node>, Sentence),
    Prefix(Token, Box<Node>)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Sentence(pub Vec<Node>);

#[macro_export]
macro_rules! sentence {
    () => (
        $crate::tree::Sentence(Vec::new())
    );
    ($($x:expr),+ $(,)?) => (
        $crate::tree::Sentence((vec![$($x),+]))
    );
}

#[cfg(test)]
mod tests {
    use crate::token::Token;
    use crate::tree::{Node, Sentence};

    #[test]
    fn empty_sentence() {
        let s = sentence![];
        assert_eq!(Sentence(vec![]), s);
    }

    #[test]
    fn nonempty_sentence() {
        let s = sentence![Node::Raw(Token::Integer(1))];
        assert_eq!(Sentence(vec![Node::Raw(Token::Integer(1))]), s);
    }
}