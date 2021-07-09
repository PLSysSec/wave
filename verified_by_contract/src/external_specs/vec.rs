use prusti_contracts::*;

#[extern_spec]
impl<T> std::vec::Vec<T> {
    #[ensures(result.len() == 0)]
    fn new() -> std::vec::Vec<T>;

    #[pure]
    fn len(&self) -> usize;

    #[ensures(self.len() == old(self.len()) + 1)]
    fn push(&mut self, value: T);

    #[ensures(self.len() == 0)]
    fn clear(&mut self);
}
