#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Encoding {
    Guess,
    Utf8,
    Cp437,
}
