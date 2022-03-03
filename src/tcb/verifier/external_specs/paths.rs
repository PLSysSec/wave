use prusti_contracts::*;

#[extern_spec]
impl std::path::PathBuf {
    #[pure]
    // #[ensures(matches!(*self, Some(_)) == result)]
    fn is_relative(&self) -> bool;

    // #[pure]
    // #[ensures(self.is_some() == !result)]
    // fn is_none(&self) -> bool;
}
