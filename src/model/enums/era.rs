pub use std::fmt;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Era {
    Early,
    Middle,
    Late,
}

impl fmt::Display for Era {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match *self {
            Era::Early => "EA",
            Era::Middle => "MA",
            Era::Late => "LA",
        };
        f.write_str(text)
    }
}
