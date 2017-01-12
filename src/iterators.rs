
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