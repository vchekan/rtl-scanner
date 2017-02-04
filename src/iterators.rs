use std::iter::*;
use std::slice::*;

pub struct Tuples<I> {
    orig: I
}

impl<I> Iterator for Tuples<I> where I: Iterator {
    type Item = (I::Item, I::Item);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(t1) = self.orig.next() {
            if let Some(t2) = self.orig.next() {
                return Some((t1, t2));
            }
        }
        return None;
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.orig.size_hint() {
            (lower, Some(upper)) => (lower, Some(upper / 2)),
            h @(_, _) => h
        }
    }
}

pub trait TuplesImpl: Sized {
    fn tuples(self) -> Tuples<Self>;
}

impl <I: Iterator> TuplesImpl for I {
    fn tuples(self) -> Tuples<Self> {
        Tuples {orig: self}
    }
}

/// Dft in general and FFTW in particular generate negative spectrum in N/2.. part of the array
pub struct DftOrder<I> {
    orig: I,
    pos: usize
}

impl<I> Iterator for DftOrder<I> where I: Iterator {
    type Item = I;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.orig.size_hint()
    }
}

/*pub fn dft_order<T>(data: &T) -> Chain<Skip<T>,Take<T>>
    where T: Iterator + ExactSizeIterator
{
    data.skip(data.len()/2).chain(data.take(data.len()/2))
}*/

pub fn dft_order<'a, T>(data: &'a [T::Item]) -> Chain<Iter<'a, T::Item>, Iter<'a, T::Item>>
    where T: Iterator
{
    data[data.len()/2+1..].iter().chain(data[..data.len()/2+1].iter())
}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn should_skip_last() {
        let a: &[f64] = &[1.,2.,3.,4.,5.];
        let mut it = a.iter().cloned().tuples();
        assert_eq!(Some((1.,2.)), it.next());
        assert_eq!(Some((3.,4.)), it.next());
        assert_eq!(None, it.next());
    }
}