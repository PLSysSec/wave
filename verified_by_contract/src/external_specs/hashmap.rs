use prusti_contracts::*;

// #[extern_spec]
// impl HashMap<SboxFd, HostFd> {
//     #[pure]
//     pub fn len(&self) -> usize;

//     #[ensures(result == None ==> self.len() == old(self.len()) + 1)]
//     pub fn insert(&mut self, key: SboxFd, value: u64) -> Option<u64>;

//     #[ensures(result == None ==> self.len() == old(self.len()) + 1)]
//     pub fn get(&mut self, key: SboxFd) -> Option<u64>;
// }
