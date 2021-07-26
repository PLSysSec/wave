use prusti_contracts::*;
use std::vec::Vec;

#[extern_spec]
impl<T> Vec<T> {
    #[ensures(result.len() == 0)]
    #[ensures(result.capacity() == 0)]
    fn new() -> Vec<T>;

    #[pure]
    fn len(&self) -> usize;

    #[ensures(self.len() == old(self.len()) + 1)]
    fn push(&mut self, value: T);

    #[ensures(self.len() == 0)]
    fn clear(&mut self);

    #[pure]
    fn capacity(&self) -> usize;

    #[ensures(self.capacity() >= old(self.len() + additional))]
    #[ensures(self.len() == old(self.len()))]
    fn reserve_exact(&mut self, additional: usize);
}
