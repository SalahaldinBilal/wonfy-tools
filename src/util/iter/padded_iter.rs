use std::iter::FusedIterator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaddingSide {
    Start,
    End,
}

#[derive(Debug, Clone)]
pub struct PaddedIter<I>
where
    I: Iterator,
    I::Item: Clone,
{
    inner: I,
    padding_item: I::Item,
    padding_remaining: usize,
    side: PaddingSide,
    inner_exhausted: bool,
}

impl<I> PaddedIter<I>
where
    I: Iterator,
    I::Item: Clone,
{
    pub fn new(inner: I, padding_item: I::Item, padding_count: usize, side: PaddingSide) -> Self {
        PaddedIter {
            inner,
            padding_item,
            padding_remaining: padding_count,
            side,
            inner_exhausted: false,
        }
    }
}

impl<I> Iterator for PaddedIter<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.side == PaddingSide::Start && self.padding_remaining > 0 {
            self.padding_remaining -= 1;
            return Some(self.padding_item.clone());
        }

        if !self.inner_exhausted {
            match self.inner.next() {
                Some(item) => return Some(item),
                None => {
                    self.inner_exhausted = true;

                    if self.side == PaddingSide::Start || self.padding_remaining == 0 {
                        return None;
                    }
                }
            }
        }

        if self.inner_exhausted && self.side == PaddingSide::End && self.padding_remaining > 0 {
            self.padding_remaining -= 1;
            return Some(self.padding_item.clone());
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (inner_low, inner_high) = self.inner.size_hint();
        // let padding_to_add = if self.inner_exhausted && self.side == PaddingSide::End {
        //     self.padding_remaining
        // } else if !self.inner_exhausted && self.side == PaddingSide::Start {
        //     self.padding_remaining
        // } else if !self.inner_exhausted && self.side == PaddingSide::End {
        //     0
        // } else {
        //     0
        // };
        let padding_to_add = if self.inner_exhausted && self.side == PaddingSide::End {
            self.padding_remaining
        } else if !self.inner_exhausted && self.side == PaddingSide::Start {
            self.padding_remaining
        } else {
            0
        };

        let low = inner_low.saturating_add(padding_to_add);
        let high = match inner_high {
            Some(h) => Some(h.saturating_add(self.padding_remaining)),
            None => None,
        };
        (low, high)
    }
}

impl<I> DoubleEndedIterator for PaddedIter<I>
where
    I: DoubleEndedIterator,
    I::Item: Clone,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.side == PaddingSide::End && self.padding_remaining > 0 {
            self.padding_remaining -= 1;
            return Some(self.padding_item.clone());
        }

        if !self.inner_exhausted {
            match self.inner.next_back() {
                Some(item) => return Some(item),
                None => {
                    self.inner_exhausted = true;

                    if self.side == PaddingSide::End || self.padding_remaining == 0 {
                        return None;
                    }
                }
            }
        }

        if self.inner_exhausted && self.side == PaddingSide::Start && self.padding_remaining > 0 {
            self.padding_remaining -= 1;
            return Some(self.padding_item.clone());
        }

        None
    }
}

impl<I> FusedIterator for PaddedIter<I>
where
    I: FusedIterator,
    I::Item: Clone,
{
}

pub trait PadExt: Iterator
where
    Self::Item: Clone,
    Self: Sized,
{
    fn pad_start(self, item: Self::Item, count: usize) -> PaddedIter<Self> {
        PaddedIter::new(self, item, count, PaddingSide::Start)
    }

    fn pad_end(self, item: Self::Item, count: usize) -> PaddedIter<Self> {
        PaddedIter::new(self, item, count, PaddingSide::End)
    }
}

impl<I> PadExt for I
where
    I: Iterator,
    <I as std::iter::Iterator>::Item: Clone,
{
}
