/// Helper macro for verbose logging, expecting the last argument(s) in braces
#[macro_export]
macro_rules! vlog {
    ($args:expr, { $($arg:tt)* }) => {
        if $args.verbose {
            eprintln!($($arg)*);
        }
    };
}

/// Helper macro for conditional verbose logging, expecting the last argument(s) in braces
#[macro_export]
macro_rules! vlog_if {
    ($args:expr, $cond:expr, { $($arg:tt)* }) => {
        if $args.verbose && $cond {
            eprintln!($($arg)*);
        }
    };
}
