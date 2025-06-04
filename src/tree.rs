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
pub struct Verse(pub Vec<Sentence>);

impl From<Verse> for Vec<Sentence> {
    fn from(verse: Verse) -> Self {
        verse.0
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
macro_rules! verse {
    () => (
        $crate::tree::Verse(Vec::new())
    );
    ($($x:expr),+ $(,)?) => (
        $crate::tree::Verse((vec![$($x),+]))
    );
}

#[cfg(test)]
mod tests {
    use crate::token::Token;
    use crate::tree::{Node, Sentence, Verse};

    #[test]
    fn empty_sentence() {
        let sentence = sentence![];
        assert_eq!(Sentence(vec![]), sentence);
    }

    #[test]
    fn nonempty_sentence() {
        let sentence = sentence![Node::Raw(Token::Integer(1))];
        assert_eq!(Sentence(vec![Node::Raw(Token::Integer(1))]), sentence);
    }
    
    #[test]
    fn vec_from_sentence() {
        let sentence = sentence![Node::Raw(Token::Integer(1))];
        let vec: Vec<_> = sentence.into();
        assert_eq!(vec![Node::Raw(Token::Integer(1))], vec);
    }

    #[test]
    fn empty_verse() {
        let verse = verse![];
        assert_eq!(Verse(vec![]), verse);
    }

    #[test]
    fn nonempty_verse() {
        let verse = verse![sentence![Node::Raw(Token::Integer(1))]];
        assert_eq!(Verse(vec![Sentence(vec![Node::Raw(Token::Integer(1))])]), verse);
    }

    #[test]
    fn vec_from_verse() {
        let verse = verse![sentence![Node::Raw(Token::Integer(1))]];
        let vec: Vec<_> = verse.into();
        assert_eq!(vec![Sentence(vec![Node::Raw(Token::Integer(1))])], vec);
    }
}