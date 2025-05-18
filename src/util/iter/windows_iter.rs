use std::collections::VecDeque;

pub trait IterWindows: Iterator {
    fn windows(self, size: usize) -> Windows<Self>
    where
        Self: Sized + Iterator,
        Self::Item: Clone,
    {
        Windows::new(self, size)
    }
}

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct Windows<I>
where
    I: Iterator,
{
    iter: I,
    size: usize,
    last: Option<VecDeque<I::Item>>,
}

impl<I> Windows<I>
where
    I: Iterator,
{
    pub fn new(iter: I, size: usize) -> Self {
        Self {
            iter,
            size,
            last: None,
        }
    }
}

impl<I> Iterator for Windows<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = VecDeque<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(new) = self.iter.next() {
            if let Some(ref mut last) = self.last {
                last.pop_back();
                last.push_front(new);
                Some(last.clone())
            } else {
                use std::iter::once;
                let iter = once(new).chain(&mut self.iter).take(self.size);
                self.last = Some(iter.collect());
                self.last.clone()
            }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut sh = self.iter.size_hint();

        if self.last.is_none() {
            let (mut low, mut hi) = sh;
            low = low.saturating_sub(self.size);
            hi = hi.map(|elt| elt.saturating_sub(self.size));
            sh = (low, hi)
        }

        sh
    }
}

impl<T> IterWindows for T where T: Iterator + ?Sized {}
