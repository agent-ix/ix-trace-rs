//! Has its own `main`: running it with libtest flags would run it, so it fails
//! on any argument other than none.

fn main() {
    if std::env::args().len() > 1 {
        eprintln!("custom main must not be given libtest flags");
        std::process::exit(1);
    }
}
