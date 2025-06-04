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

impl From<Sentence> for Vec<Node> {
    fn from(sentence: Sentence) -> Self {
        sentence.0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Stanza(pub Vec<Sentence>);

impl From<Stanza> for Vec<Sentence> {
    fn from(stanza: Stanza) -> Self {
        stanza.0
    }
}

#[macro_export]
macro_rules! sentence {
    () => (
        $crate::tree::Sentence(Vec::new())
    );
    ($($x:expr),+ $(,)?) => (
        $crate::tree::Sentence((vec![$($x),+]))
    );
}

#[macro_export]
macro_rules! stanza {
    () => (
        $crate::tree::Stanza(Vec::new())
    );
    ($($x:expr),+ $(,)?) => (
        $crate::tree::Stanza((vec![$($x),+]))
    );
}

#[cfg(test)]
mod tests {
    use crate::token::Token;
    use crate::tree::{Node, Sentence, Stanza};

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
    
    #[test]
    fn vec_from_sentence() {
        let s = sentence![Node::Raw(Token::Integer(1))];
        let v: Vec<_> = s.into();
        assert_eq!(vec![Node::Raw(Token::Integer(1))], v);
    }

    #[test]
    fn empty_stanza() {
        let s = stanza![];
        assert_eq!(Stanza(vec![]), s);
    }

    #[test]
    fn nonempty_stanza() {
        let s = stanza![sentence![Node::Raw(Token::Integer(1))]];
        assert_eq!(Stanza(vec![Sentence(vec![Node::Raw(Token::Integer(1))])]), s);
    }

    #[test]
    fn vec_from_stanza() {
        let s = stanza![sentence![Node::Raw(Token::Integer(1))]];
        let v: Vec<_> = s.into();
        assert_eq!(vec![Sentence(vec![Node::Raw(Token::Integer(1))])], v);
    }
}