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
pub struct Phrase<'a>(Vec<Node<'a>>, Metadata);

impl<'a> Phrase<'a> {
    #[inline]
    pub fn new(nodes: Vec<Node<'a>>, metadata: Metadata) -> Self {
        assert!(!nodes.is_empty(), "phrase must comprise at least one node");
        Self(nodes, metadata)
    }

    #[inline]
    pub fn into_nodes(self) -> Vec<Node<'a>> {
        self.0
    }
    
    pub fn metadata(&self) -> &Metadata {
        &self.1
    }
}

impl<'a> From<Phrase<'a>> for Vec<Node<'a>> {
    fn from(phrase: Phrase<'a>) -> Self {
        phrase.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verse<'a>(Vec<Phrase<'a>>);

impl<'a> Verse<'a> {
    #[inline]
    pub fn new(phrases: Vec<Phrase<'a>>) -> Self {
        // assert!(!phrases.is_empty(), "verse must comprise at least one phrase");
        Self(phrases)
    }

    #[inline]
    pub fn into_phrases(self) -> Vec<Phrase<'a>> {
        self.0
    }

    #[inline]
    pub fn flatten(self) -> impl Iterator<Item = Node<'a>> {
        self.0.into_iter().flat_map(|phrase| phrase.0.into_iter())
    }
    
    pub fn metadata(&self) -> Metadata {
        if self.0.is_empty() {
            Metadata::unspecified()
        } else {
            let start = self.0[0].metadata().start.clone();
            let end = self.0[self.0.len() - 1].metadata().end.clone();
            Metadata { start, end }
        }
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
        $crate::tree::Phrase::new(Vec::new(), $crate::metadata::Metadata::unspecified())
    );
    ($($x:expr),+ $(,)?) => (
        $crate::tree::Phrase::new((vec![$($x),+]), $crate::metadata::Metadata::unspecified())
    );
}

#[macro_export]
macro_rules! verse {
    () => (
        $crate::tree::Verse::new(Vec::new())
    );
    ($($x:expr),+ $(,)?) => (
        $crate::tree::Verse::new((vec![$($x),+]))
    );
}

#[cfg(test)]
mod tests {
    use crate::metadata::Metadata;
    use crate::token::Token;
    use crate::tree::{Node, Phrase, Verse};

    #[test]
    #[should_panic(expected = "phrase must comprise at least one node")]
    fn empty_phrase_err() {
        let _ = phrase![];
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