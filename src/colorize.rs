use std::fmt::Display;

pub enum Colorize<T: Display> {
    Cyan(T),
    Green(T),
    Purple(T),
    Magenta(T),
}

impl<T: Display> Display for Colorize<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (start_code, content) = match self {
            Colorize::Cyan(content) => ("\x1b[36m", content),
            Colorize::Purple(content) => ("\x1b[35m", content),
            Colorize::Green(content) => ("\x1b[32m", content),
            Colorize::Magenta(content) => ("\x1b[35;1m", content),
        };

        // From https://doc.rust-lang.org/nightly/std/fmt/struct.Formatter.html#method.precision
        if let Some(precision) = f.precision() {
            write!(f, "{}{:.*}\x1b[0m", start_code, precision, content)
        } else {
            write!(f, "{}{}\x1b[0m", start_code, content)
        }
    }
}
