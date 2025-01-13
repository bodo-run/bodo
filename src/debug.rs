use std::sync::atomic::{AtomicBool, Ordering};

static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn set_verbose(verbose: bool) {
    VERBOSE.store(verbose, Ordering::SeqCst);
}

pub fn is_verbose() -> bool {
    VERBOSE.load(Ordering::SeqCst)
}

pub fn debug(msg: &str) {
    if is_verbose() {
        eprintln!("[DEBUG] {}", msg);
    }
}

pub fn debug_fmt(args: std::fmt::Arguments<'_>) {
    if is_verbose() {
        eprintln!("[DEBUG] {}", args);
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::debug::debug_fmt(format_args!($($arg)*))
    };
}
