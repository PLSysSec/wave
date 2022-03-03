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
    #[ensures(self.capacity() >= old(self.capacity()))]
    fn push(&mut self, value: T);

    #[ensures(self.len() == 0)]
    fn clear(&mut self);

    #[pure]
    fn capacity(&self) -> usize;

    #[ensures(self.capacity() >= old(self.len() + additional))]
    #[ensures(self.len() == old(self.len()))]
    fn reserve_exact(&mut self, additional: usize);

    // #[pure]
    // fn as_slice(&self) -> &[T];

    // #[pure]
    // fn as_mut_slice(&mut self) -> &mut [T];

    // #[pure]
    // #[requires (index < MAX_SBOX_FDS )]
    // // #[requires(0 <= index && index < self.len())]
    // // #[ensures(*result == old(self.lookup(index)))]
    // pub fn get(&self, index: usize) -> &T {
    //     self.get(index).unwrap()
    // }
}
