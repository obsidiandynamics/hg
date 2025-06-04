use crate::token::Token;

#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    Raw(Token),
    List(Vec<Verse>),
    Container(Verse),
    Cons(Box<Node>, Phrase),
    Prefix(Token, Box<Node>)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Phrase(pub Vec<Node>);

impl From<Phrase> for Vec<Node> {
    fn from(phrase: Phrase) -> Self {
        phrase.0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Verse(pub Vec<Phrase>);

impl From<Verse> for Vec<Phrase> {
    fn from(verse: Verse) -> Self {
        verse.0
    }
}

#[macro_export]
macro_rules! phrase {
    () => (
        $crate::tree::Phrase(Vec::new())
    );
    ($($x:expr),+ $(,)?) => (
        $crate::tree::Phrase((vec![$($x),+]))
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
    use crate::tree::{Node, Phrase, Verse};

    #[test]
    fn empty_phrase() {
        let phrase = phrase![];
        assert_eq!(Phrase(vec![]), phrase);
    }

    #[test]
    fn nonempty_phrase() {
        let phrase = phrase![Node::Raw(Token::Integer(1))];
        assert_eq!(Phrase(vec![Node::Raw(Token::Integer(1))]), phrase);
    }
    
    #[test]
    fn vec_from_phrase() {
        let phrase = phrase![Node::Raw(Token::Integer(1))];
        let vec: Vec<_> = phrase.into();
        assert_eq!(vec![Node::Raw(Token::Integer(1))], vec);
    }

    #[test]
    fn empty_verse() {
        let verse = verse![];
        assert_eq!(Verse(vec![]), verse);
    }

    #[test]
    fn nonempty_verse() {
        let verse = verse![phrase![Node::Raw(Token::Integer(1))]];
        assert_eq!(Verse(vec![Phrase(vec![Node::Raw(Token::Integer(1))])]), verse);
    }

    #[test]
    fn vec_from_verse() {
        let verse = verse![phrase![Node::Raw(Token::Integer(1))]];
        let vec: Vec<_> = verse.into();
        assert_eq!(vec![Phrase(vec![Node::Raw(Token::Integer(1))])], vec);
    }
}