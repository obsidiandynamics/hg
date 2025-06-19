use crate::lexer::Fragment;

pub struct FragmentStream<'a, I: Iterator<Item=Fragment<'a>>> {
    iter: I,
    stashed_fragment: Option<Fragment<'a>>
}

impl<'a, I: Iterator<Item=Fragment<'a>>> FragmentStream<'a, I> {
    #[inline(always)]
    pub fn stash(&mut self, fragment: Fragment<'a>) {
        debug_assert!(self.stashed_fragment.is_none());
        self.stashed_fragment = Some(fragment);
    }
}

impl<'a, I: Iterator<Item=Fragment<'a>>> From<I> for FragmentStream<'a, I> {
    #[inline(always)]
    fn from(iter: I) -> Self {
        Self {
            iter, stashed_fragment: None
        }
    }
}

impl<'a, I: Iterator<Item=Fragment<'a>>> Iterator for FragmentStream<'a, I> {
    type Item = Fragment<'a>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.stashed_fragment.take().or_else(|| self.iter.next())
    }
}