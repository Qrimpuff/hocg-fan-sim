use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

// This is a bare-bones implementation. A real library would provide additional
// information in its error type, for example the line and column at which the
// error occurred, the byte offset into the input, or the current key being
// processed.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    // One or more variants that can be created by data structures through the
    // `ser::Error` and `de::Error` traits. For example the Serialize impl for
    // Mutex<T> might return an error because the mutex is poisoned, or the
    // Deserialize impl for a struct may return an error because a required
    // field is missing.
    Message(String),

    // Zero or more variants that can be created directly by the Serializer and
    // Deserializer without going through `ser::Error` and `de::Error`. These
    // are specific to the format, in this case JSON.
    ExpectedString,
    ExpectedToken,
    MissingBracket,
    UnbalancedBrackets,
    NoTokens,
    RemainingTokens,
    UnexpectedToken(String, String),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::ExpectedString => formatter.write_str("Expected string"),
            Error::ExpectedToken => formatter.write_str("Expected token"),
            Error::MissingBracket => formatter.write_str("Missing bracket"),
            Error::UnbalancedBrackets => formatter.write_str("Unbalanced brackets"),
            Error::NoTokens => formatter.write_str("No tokens"),
            Error::RemainingTokens => formatter.write_str("Remaining tokens"),
            Error::UnexpectedToken(expected, unexpected) => {
                write!(
                    formatter,
                    r#"Unexpected token: "{unexpected}", expected <{expected}>"#
                )
            } /* and so forth */
        }
    }
}

impl std::error::Error for Error {}
