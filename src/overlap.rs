use std::{
    iter::{Chain, Map, RepeatN, Rev},
    ops::{Add, Deref},
    str::FromStr,
};

use crate::error::{UnknownError, unknown_error_expected};

use std::iter::FusedIterator;

// unused for now

pub fn reflect_iter_map<I: Iterator + Clone + DoubleEndedIterator<Item = B>, F, B>(
    iter: I,
    f: F,
) -> Chain<Map<Rev<I>, F>, I>
where
    F: FnMut(I::Item) -> B,
{
    let mut cloned_iter = iter.clone();
    cloned_iter.next();

    cloned_iter.rev().map(f).chain(iter)
}
