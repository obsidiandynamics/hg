use crate::metadata::Metadata;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node<'a> {
    Raw(Token<'a>, Metadata),
    List(Vec<Verse<'a>>, Metadata),
    Relation(Box<Node<'a>>, Phrase<'a>, Metadata),
}

impl Node<'_> {
    #[inline]
    pub fn metadata(&self) -> &Metadata {
        match self {
            Node::Raw(_, metadata) => metadata,
            Node::List(_, metadata) => metadata,
            Node::Relation(_, _, metadata) => metadata,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phrase<'a>(pub Vec<Node<'a>>, pub Metadata);

impl<'a> From<Phrase<'a>> for Vec<Node<'a>> {
    fn from(phrase: Phrase<'a>) -> Self {
        phrase.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verse<'a>(pub Vec<Phrase<'a>>);

impl<'a> Verse<'a> {
    pub fn flatten(self) -> impl Iterator<Item = Node<'a>> {
        self.0.into_iter().flat_map(|phrase| phrase.0.into_iter())
    }
}

impl<'a> From<Verse<'a>> for Vec<Phrase<'a>> {
    fn from(verse: Verse<'a>) -> Self {
        verse.0
    }
}

#[macro_export]
macro_rules! phrase {
    () => (
        $crate::tree::Phrase(Vec::new(), $crate::metadata::Metadata::unspecified())
    );
    ($($x:expr),+ $(,)?) => (
        $crate::tree::Phrase((vec![$($x),+]), $crate::metadata::Metadata::unspecified())
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
        assert_eq!(Phrase(vec![], Metadata::unspecified()), phrase);
    }

    #[test]
    fn nonempty_phrase() {
        let phrase = phrase![Node::Raw(Token::Integer(1), Metadata::unspecified())];
        assert_eq!(Phrase(vec![Node::Raw(Token::Integer(1), Metadata::unspecified())], Metadata::unspecified()), phrase);
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
        assert_eq!(Verse(vec![Phrase(vec![Node::Raw(Token::Integer(1), Metadata::unspecified())], Metadata::unspecified())]), verse);
    }

    #[test]
    fn vec_from_verse() {
        let verse = verse![phrase![Node::Raw(Token::Integer(1), Metadata::unspecified())]];
        let vec: Vec<_> = verse.into();
        assert_eq!(vec![Phrase(vec![Node::Raw(Token::Integer(1), Metadata::unspecified())], Metadata::unspecified())], vec);
    }
}