use crate::metadata::Metadata;
use crate::token::Token;

#[derive(Debug, PartialEq, Eq)]
pub enum Node<'a> {
    Raw(Token<'a>, Metadata),
    List(Vec<Verse<'a>>, Metadata),
    Cons(Box<Node<'a>>, Phrase<'a>, Metadata),
    Prefix(Token<'a>, Box<Node<'a>>, Metadata)
}

impl Node<'_> {
    #[inline]
    pub fn metadata(&self) -> &Metadata {
        match self {
            Node::Raw(_, metadata) => metadata,
            Node::List(_, metadata) => metadata,
            Node::Cons(_, _, metadata) => metadata,
            Node::Prefix(_, _, metadata) => metadata,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Phrase<'a>(pub Vec<Node<'a>>);

impl<'a> From<Phrase<'a>> for Vec<Node<'a>> {
    fn from(phrase: Phrase<'a>) -> Self {
        phrase.0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Verse<'a>(pub Vec<Phrase<'a>>);

impl<'a> From<Verse<'a>> for Vec<Phrase<'a>> {
    fn from(verse: Verse<'a>) -> Self {
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
    use crate::metadata::Metadata;
    use crate::token::Token;
    use crate::tree::{Node, Phrase, Verse};

    #[test]
    fn empty_phrase() {
        let phrase = phrase![];
        assert_eq!(Phrase(vec![]), phrase);
    }

    #[test]
    fn nonempty_phrase() {
        let phrase = phrase![Node::Raw(Token::Integer(1), Metadata::unspecified())];
        assert_eq!(Phrase(vec![Node::Raw(Token::Integer(1), Metadata::unspecified())]), phrase);
    }
    
    #[test]
    fn vec_from_phrase() {
        let phrase = phrase![Node::Raw(Token::Integer(1), Metadata::unspecified())];
        let vec: Vec<_> = phrase.into();
        assert_eq!(vec![Node::Raw(Token::Integer(1), Metadata::unspecified())], vec);
    }

    #[test]
    fn empty_verse() {
        let verse = verse![];
        assert_eq!(Verse(vec![]), verse);
    }

    #[test]
    fn nonempty_verse() {
        let verse = verse![phrase![Node::Raw(Token::Integer(1), Metadata::unspecified())]];
        assert_eq!(Verse(vec![Phrase(vec![Node::Raw(Token::Integer(1), Metadata::unspecified())])]), verse);
    }

    #[test]
    fn vec_from_verse() {
        let verse = verse![phrase![Node::Raw(Token::Integer(1), Metadata::unspecified())]];
        let vec: Vec<_> = verse.into();
        assert_eq!(vec![Phrase(vec![Node::Raw(Token::Integer(1), Metadata::unspecified())])], vec);
    }
}