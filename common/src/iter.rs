pub struct PairIterator<'a, T> {
    items: &'a [T],
    i: usize,
    j: usize,
}

impl<'a, T> PairIterator<'a, T> {
    pub fn new(items: &'a [T]) -> Self {
        Self { items, i: 0, j: 1 }
    }
}

impl<'a, T> Iterator for PairIterator<'a, T> {
    type Item = (&'a T, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.items.len() - 1 {
            return None;
        }

        let pair = (&self.items[self.i], &self.items[self.j]);

        if self.j < self.items.len() - 1 {
            self.j += 1;
        } else {
            self.i += 1;
            self.j = self.i + 1;
        }

        Some(pair)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use heapless::Vec;

    #[test]
    fn test_pair_iterator() {
        let items = [1, 2, 3];
        let pairs: Vec<_, 3> = PairIterator::new(&items).collect();
        assert_eq!(&pairs[..], &[(&1, &2), (&1, &3), (&2, &3)]);
    }
}
