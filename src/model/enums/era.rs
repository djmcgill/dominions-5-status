use std::fmt;
use enum_primitive_derive::Primitive;

#[derive(Clone, Copy, PartialEq, Debug, Primitive)]
pub enum Era {
    Early = 0,
    Middle = 1,
    Late = 2,
}

impl Era {
    pub fn from_string(string: &str) -> Option<Era> {
        match string.to_uppercase().as_ref() {
            "EA" => Some(Era::Early),
            "MA" => Some(Era::Middle),
            "LA" => Some(Era::Late),
            _ => None,
        }
    }
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
