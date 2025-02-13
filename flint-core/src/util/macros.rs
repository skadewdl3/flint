#[macro_export]
macro_rules! ui {
    (<column> $($child:tt)* </column>) => {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![$($child),*].as_ref().into())
    };
}
