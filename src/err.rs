#[derive(Debug)]
pub enum Error {
    Socket(tokio_tungstenite::tungstenite::Error),
    Message(MessageError),
    Io(std::io::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Socket(err) => write!(f, "{err}"),
            Self::Message(err) => write!(f, "{err}"),
            Self::Io(err) => write!(f, "{err}"),
        }
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(value: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::Socket(value)
    }
}

impl From<MessageError> for Error {
    fn from(value: MessageError) -> Self {
        Self::Message(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug)]
pub enum MessageError {
    InvalidCode,
    NoMoreDataExpected,
    MoreDataExpected(&'static str),
    InvalidTyp,
    CounterParseError,
    KeyCodeParseError,
    UnexpectedMessageTyp,
}

impl std::error::Error for MessageError {}

impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCode => write!(f, "Invalid Message Code"),
            Self::NoMoreDataExpected => write!(f, "No more data expected"),
            Self::MoreDataExpected(missing) => write!(f, "More data expected: {missing}"),
            Self::InvalidTyp => write!(f, "Invalid event typ"),
            Self::CounterParseError => write!(f, "Failed to parse ping counter"),
            Self::KeyCodeParseError => write!(f, "Failed to parse key code"),
            Self::UnexpectedMessageTyp => write!(f, "Unexpected Message Typ"),
        }
    }
}
