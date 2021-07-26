use prusti_contracts::*;

#[extern_spec]
impl<T> std::option::Option<T> {
    #[pure]
    #[ensures(matches!(*self, Some(_)) == result)]
    fn is_some(&self) -> bool;

    #[pure]
    #[ensures(self.is_some() == !result)]
    fn is_none(&self) -> bool;

    #[requires(self.is_some())]
    fn unwrap(self) -> T;

    // ...
}
