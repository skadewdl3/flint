use std::sync::atomic::AtomicBool;

pub struct Flags {
    pub non_interactive: AtomicBool,
}

// Create a static global instance
pub static GLOBAL_FLAGS: Flags = Flags {
    non_interactive: AtomicBool::new(false),
};

#[macro_export]
macro_rules! get_flag {
    ($name:ident) => {{
        use std::sync::atomic::Ordering;
        use $crate::util::flags::GLOBAL_FLAGS;

        GLOBAL_FLAGS.$name.load(Ordering::SeqCst)
    }};
}

#[macro_export]
macro_rules! set_flag {
    ($name:ident, $value:expr) => {{
        use std::sync::atomic::Ordering;
        use $crate::util::flags::GLOBAL_FLAGS;

        GLOBAL_FLAGS.$name.store($value, Ordering::SeqCst)
    }};
}
