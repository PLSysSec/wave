use owned_components::*;
use prusti_contracts::*;

// #[extern_spec]
// impl std::path::PathBuf {
//     #[pure]
//     // #[ensures(matches!(*self, Some(_)) == result)]
//     fn is_relative(&self) -> bool;

//     // #[pure]
//     // #[ensures(self.is_some() == !result)]
//     // fn is_none(&self) -> bool;
// }

// #[extern_spec]
// impl OwnedComponents {
//     #[pure]
//     fn len(&self) -> usize;

//     // #[ensures(result.len() == 0)]
//     pub fn new() -> OwnedComponents;

//     //pub fn as_path(&self) -> &Path;

//     //pub fn parse(p: PathBuf) -> Self;

//     // #[pure]
//     // #[requires(idx < self.len())]
//     pub fn lookup(&self, idx: usize);

//     // #[ensures(self.len() == old(self.len()) + 1)]
//     // #[ensures(self.lookup(old(self.len())) == old(value))]
//     // #[ensures(forall(|i: usize| (i < old(self.len())) ==>
//     //                 self.lookup(i) == old(self.lookup(i))))]
//     pub fn push(&mut self, value: OwnedComponent);

//     // #[requires(self.len() > 0)]
//     // #[ensures(self.len() == old(self.len()) - 1)]
//     // #[ensures(forall(|i: usize| (i < self.len()) ==>
//     //                 self.lookup(i) == old(self.lookup(i))))]
//     pub fn pop(&mut self);

//     pub fn unparse(self) -> Vec<u8>;

// }
