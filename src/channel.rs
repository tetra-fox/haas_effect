use nih_plug::params::enums::Enum;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Enum, enum_iterator::Sequence)]
pub enum Channel {
    Left,
    Right,
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match *self {
            Channel::Left => "Left",
            Channel::Right => "Right",
        };
        write!(f, "{}", str)
    }
}
