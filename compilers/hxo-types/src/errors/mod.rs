use crate::Span;
use std::fmt;

#[derive(Debug)]
pub struct Error {
    pub kind: Box<ErrorKind>,
}

#[derive(Debug)]
pub enum ErrorKind {
    Io(std::io::Error),
    Parse { message: String, span: Span },
    UnexpectedChar { character: char, span: Span },
    ExpectedChar { expected: char, found: char, span: Span },
    ExpectedString { expected: String, found: String, span: Span },
    ExpectedOneOf { expected: Vec<String>, found: String, span: Span },
    ExpectedClosingTag { expected: String, found: String, span: Span },
    InvalidI18n { format: String, span: Span },
    UnsupportedI18nFormat { format: String, span: Span },
    NotImplemented { feature: String, span: Span },
    ParseFloatError { source: String, span: Span },
    UnexpectedContent { message: String, span: Span },
    TrailingContent { span: Span },
    ExternalError { source: String, details: String, span: Span },
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind: Box::new(kind) }
    }

    pub fn io_error(err: std::io::Error) -> Self {
        Self::new(ErrorKind::Io(err))
    }

    pub fn unexpected_char(character: char, span: Span) -> Self {
        Self::new(ErrorKind::UnexpectedChar { character, span })
    }

    pub fn expected_char(expected: char, found: char, span: Span) -> Self {
        Self::new(ErrorKind::ExpectedChar { expected, found, span })
    }

    pub fn expected_string(expected: String, found: String, span: Span) -> Self {
        Self::new(ErrorKind::ExpectedString { expected, found, span })
    }

    pub fn expected_one_of(expected: Vec<String>, found: String, span: Span) -> Self {
        Self::new(ErrorKind::ExpectedOneOf { expected, found, span })
    }

    pub fn expected_closing_tag(expected: String, found: String, span: Span) -> Self {
        Self::new(ErrorKind::ExpectedClosingTag { expected, found, span })
    }

    pub fn invalid_i18n(format: String, span: Span) -> Self {
        Self::new(ErrorKind::InvalidI18n { format, span })
    }

    pub fn unsupported_i18n_format(format: String, span: Span) -> Self {
        Self::new(ErrorKind::UnsupportedI18nFormat { format, span })
    }

    pub fn not_implemented(feature: String, span: Span) -> Self {
        Self::new(ErrorKind::NotImplemented { feature, span })
    }

    pub fn parse_error(message: String, span: Span) -> Self {
        Self::new(ErrorKind::Parse { message, span })
    }

    pub fn parse_float_error(source: String, span: Span) -> Self {
        Self::new(ErrorKind::ParseFloatError { source, span })
    }

    pub fn trailing_content(span: Span) -> Self {
        Self::new(ErrorKind::TrailingContent { span })
    }

    pub fn external_error(source: String, details: String, span: Span) -> Self {
        Self::new(ErrorKind::ExternalError { source, details, span })
    }

    pub fn span(&self) -> Span {
        self.kind.span()
    }
}

impl ErrorKind {
    pub fn span(&self) -> Span {
        match self {
            ErrorKind::Io(_) => Span::unknown(),
            ErrorKind::Parse { span, .. } => *span,
            ErrorKind::UnexpectedChar { span, .. } => *span,
            ErrorKind::ExpectedChar { span, .. } => *span,
            ErrorKind::ExpectedString { span, .. } => *span,
            ErrorKind::ExpectedOneOf { span, .. } => *span,
            ErrorKind::ExpectedClosingTag { span, .. } => *span,
            ErrorKind::InvalidI18n { span, .. } => *span,
            ErrorKind::UnsupportedI18nFormat { span, .. } => *span,
            ErrorKind::NotImplemented { span, .. } => *span,
            ErrorKind::ParseFloatError { span, .. } => *span,
            ErrorKind::UnexpectedContent { span, .. } => *span,
            ErrorKind::TrailingContent { span, .. } => *span,
            ErrorKind::ExternalError { span, .. } => *span,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self.kind {
            ErrorKind::Io(err) => write!(f, "IO error: {}", err),
            ErrorKind::Parse { message, span } => {
                write!(f, "Parse error: {}", message)?;
                if !span.is_unknown() {
                    write!(f, " at {}:{}", span.start.line, span.start.column)?;
                }
                Ok(())
            }
            ErrorKind::UnexpectedChar { character, span } => {
                write!(f, "Unexpected character '{}'", character)?;
                if !span.is_unknown() {
                    write!(f, " at {}:{}", span.start.line, span.start.column)?;
                }
                Ok(())
            }
            ErrorKind::ExpectedChar { expected, found, span } => {
                write!(f, "Expected '{}', found '{}'", expected, found)?;
                if !span.is_unknown() {
                    write!(f, " at {}:{}", span.start.line, span.start.column)?;
                }
                Ok(())
            }
            ErrorKind::ExpectedString { expected, found, span } => {
                write!(f, "Expected '{}', found '{}'", expected, found)?;
                if !span.is_unknown() {
                    write!(f, " at {}:{}", span.start.line, span.start.column)?;
                }
                Ok(())
            }
            ErrorKind::ExpectedOneOf { expected, found, span } => {
                write!(f, "Expected one of {:?}, found '{}'", expected, found)?;
                if !span.is_unknown() {
                    write!(f, " at {}:{}", span.start.line, span.start.column)?;
                }
                Ok(())
            }
            ErrorKind::ExpectedClosingTag { expected, found, span } => {
                write!(f, "Expected closing tag </{}>, found </{}>", expected, found)?;
                if !span.is_unknown() {
                    write!(f, " at {}:{}", span.start.line, span.start.column)?;
                }
                Ok(())
            }
            ErrorKind::InvalidI18n { format, span } => {
                write!(f, "Invalid i18n {}", format)?;
                if !span.is_unknown() {
                    write!(f, " at {}:{}", span.start.line, span.start.column)?;
                }
                Ok(())
            }
            ErrorKind::UnsupportedI18nFormat { format, span } => {
                write!(f, "Unsupported i18n format: {}", format)?;
                if !span.is_unknown() {
                    write!(f, " at {}:{}", span.start.line, span.start.column)?;
                }
                Ok(())
            }
            ErrorKind::NotImplemented { feature, span } => {
                write!(f, "{} not yet implemented", feature)?;
                if !span.is_unknown() {
                    write!(f, " at {}:{}", span.start.line, span.start.column)?;
                }
                Ok(())
            }
            ErrorKind::ParseFloatError { source, span } => {
                write!(f, "Failed to parse float: '{}'", source)?;
                if !span.is_unknown() {
                    write!(f, " at {}:{}", span.start.line, span.start.column)?;
                }
                Ok(())
            }
            ErrorKind::UnexpectedContent { message, span } => {
                write!(f, "Unexpected content: {}", message)?;
                if !span.is_unknown() {
                    write!(f, " at {}:{}", span.start.line, span.start.column)?;
                }
                Ok(())
            }
            ErrorKind::TrailingContent { span } => {
                write!(f, "Unexpected trailing content")?;
                if !span.is_unknown() {
                    write!(f, " at {}:{}", span.start.line, span.start.column)?;
                }
                Ok(())
            }
            ErrorKind::ExternalError { source, details, span } => {
                write!(f, "Error in {}: {}", source, details)?;
                if !span.is_unknown() {
                    write!(f, " at {}:{}", span.start.line, span.start.column)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &*self.kind {
            ErrorKind::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::new(ErrorKind::Io(err))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
