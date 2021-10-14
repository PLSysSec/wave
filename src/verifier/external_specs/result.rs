use prusti_contracts::*;

#[extern_spec]
impl<T, E> std::result::Result<T, E> {
    #[pure]
    #[ensures(matches!(*self, Ok(_)) == result)]
    fn is_ok(&self) -> bool;

    #[pure]
    #[ensures(self.is_ok() == !result)]
    fn is_err(&self) -> bool;

    // #[requires(self.is_some())]
    // fn unwrap(self) -> T;

    // ...
}
