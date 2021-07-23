use prusti_contracts::*;
use std::vec::Vec;
// use std::slice::SliceIndex;


#[extern_spec]
impl<T> Vec<T> {
    #[ensures(result.len() == 0)]
    fn new() -> Vec<T>;

    #[pure]
    fn len(&self) -> usize;

    #[ensures(self.len() == old(self.len()) + 1)]
    fn push(&mut self, value: T);

    #[ensures(self.len() == 0)]
    fn clear(&mut self);
}
