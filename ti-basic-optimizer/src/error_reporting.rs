use ariadne;
use titokens::tokenizer::TokenBoundaries;

macro_rules! next_or_err {
    ($tokens: ident) => {
        $tokens.next().ok_or_else(|| {
            (crate::error_reporting::LineReport::new(
                $tokens.current_position(),
                "Unexpected end of input.",
                None,
            )
            .with_label($tokens.current_position(), "here"))
        })
    };

    ($tokens: ident, $message: literal) => {
        $tokens.next().ok_or_else(|| {
            (crate::error_reporting::LineReport::new($tokens.current_position(), $message, None)
                .with_label($tokens.current_position(), "here"))
        })
    };
}

macro_rules! expect_tok {
    ($tokens: ident, $token: expr, $token_name: literal) => {
        crate::error_reporting::next_or_err!($tokens).and_then(|tok| {
            if tok != ($token) {
                Err(crate::error_reporting::LineReport::new(
                    $tokens.current_position(),
                    concat!("Expected token \"", $token_name, "\"."),
                    Some("Add the token."),
                ))
            } else {
                Ok(())
            }
        })
    };

    ($tokens: ident, $token: expr, $error: literal, $help: literal) => {
        crate::error_reporting::next_or_err!($tokens).and_then(|tok| {
            if tok != ($token) {
                Err(crate::error_reporting::LineReport::new(
                    $tokens.current_position(),
                    $error,
                    Some($help),
                ))
            } else {
                Ok(())
            }
        })
    };
}

macro_rules! expect_some {
    ($option: expr, $tokens: ident, $expected_kind: literal) => {
        $option.ok_or_else(|| {
            crate::error_reporting::LineReport::new(
                $tokens.current_position(),
                concat!("Expected to find ", $expected_kind, "."),
                None,
            )
            .with_label(
                $tokens.current_position(),
                concat!("This was not parsed as ", $expected_kind, "."),
            )
        })
    };

    ($option: expr, $tokens: ident, $expected_kind: literal, $help: literal) => {
        $option.ok_or_else(|| {
            crate::error_reporting::LineReport::new(
                $tokens.current_position(),
                concat!("Expected to find ", $expected_kind, "."),
                None,
            )
            .with_label($tokens.current_position(), concat!($help))
        })
    };
}

pub(crate) use expect_some;
pub(crate) use expect_tok;
pub(crate) use next_or_err;

#[derive(Debug, Clone)]
pub struct LineReport {
    location: usize,
    message: String,
    suggestion: Option<String>,

    labels: Vec<(std::ops::Range<usize>, String)>,
}

impl LineReport {
    #[must_use]
    pub fn new(location: usize, message: &str, suggestion: Option<&str>) -> Self {
        LineReport {
            location,
            message: message.to_string(),
            suggestion: suggestion.map(|x| x.to_string()),

            labels: vec![],
        }
    }

    #[must_use]
    pub fn with_span_label(mut self, location: std::ops::Range<usize>, message: &str) -> Self {
        self.labels.push((location, message.to_string()));

        self
    }

    #[must_use]
    pub fn with_label(mut self, location: usize, message: &str) -> Self {
        self.labels.push((location..location, message.to_string()));

        self
    }

    pub fn error(self, boundaries: TokenBoundaries) {
        let mut builder = ariadne::Report::build(
            ariadne::ReportKind::Error,
            (),
            boundaries.single(self.location).start,
        )
        .with_message(self.message);

        if self.labels.is_empty() {
            builder = builder.with_label(
                ariadne::Label::new(boundaries.single(self.location)).with_message("here"),
            );
        } else {
            builder = builder.with_labels(self.labels.iter().map(|(range, message)| {
                ariadne::Label::new(boundaries.range(range.start..range.end)).with_message(message)
            }))
        }

        if let Some(help) = self.suggestion {
            builder = builder.with_help(help);
        }

        builder
            .finish()
            .eprint(ariadne::Source::from(boundaries.to_string()))
            .unwrap();
    }
}
