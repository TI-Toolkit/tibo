use ariadne;
use std::ops::Range;
use titokens::tokenizer::TokenBoundaries;

macro_rules! next_or_err {
    ($tokens: ident) => {
        $tokens.next().ok_or_else(|| {
            (crate::error_reporting::LineReport::new(
                $tokens.current_position() - 2,
                "Unexpected end of input.",
                None,
            )
            .with_label($tokens.current_position() - 2, "here"))
        })
    };

    ($tokens: ident, $message: literal) => {
        $tokens.next().ok_or_else(|| {
            (crate::error_reporting::LineReport::new(
                $tokens.current_position() - 2,
                $message,
                None,
            )
            .with_label($tokens.current_position() - 2, "here"))
        })
    };
}

macro_rules! expect_tok {
    ($tokens: ident, $token: expr, $token_name: literal) => {
        crate::error_reporting::next_or_err!($tokens).and_then(|tok| {
            if tok != ($token) {
                Err(crate::error_reporting::LineReport::new(
                    $tokens.current_position() - 1,
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
                    $tokens.current_position() - 1,
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
                $tokens.current_position() - 1,
                concat!("Expected to find ", $expected_kind, "."),
                None,
            )
            .with_label(
                $tokens.current_position() - 1,
                concat!("This was not parsed as ", $expected_kind, "."),
            )
        })
    };

    ($option: expr, $tokens: ident, $expected_kind: literal, $help: literal) => {
        $option.ok_or_else(|| {
            crate::error_reporting::LineReport::new(
                $tokens.current_position() - 1,
                concat!("Expected to find ", $expected_kind, "."),
                None,
            )
            .with_label($tokens.current_position() - 1, concat!($help))
        })
    };

    ($option: expr, $tokens: ident, $ofs: expr, $expected_kind: literal, $help: literal) => {
        $option.ok_or_else(|| {
            crate::error_reporting::LineReport::new(
                $tokens.current_position() - $ofs,
                concat!("Expected to find ", $expected_kind, "."),
                None,
            )
            .with_label($tokens.current_position() - $ofs, concat!($help))
        })
    };
}

pub(crate) use expect_some;
pub(crate) use expect_tok;
pub(crate) use next_or_err;

#[derive(Debug, Clone)]
enum LabelKind {
    Single(usize),
    Span(Range<usize>),
}

impl LabelKind {
    pub fn string_indices(&self, token_boundaries: &TokenBoundaries) -> Range<usize> {
        match self {
            LabelKind::Single(tok_idx) => token_boundaries.single(*tok_idx),
            LabelKind::Span(tok_range) => token_boundaries.range(tok_range.clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LineReport {
    location: usize,
    message: String,
    suggestion: Option<String>,

    labels: Vec<(LabelKind, String)>,
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
        self.labels
            .push((LabelKind::Span(location), message.to_string()));

        self
    }

    #[must_use]
    pub fn with_label(mut self, location: usize, message: &str) -> Self {
        self.labels
            .push((LabelKind::Single(location), message.to_string()));

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
            builder = builder.with_labels(self.labels.iter().map(|(label_kind, message)| {
                ariadne::Label::new(label_kind.string_indices(&boundaries)).with_message(message)
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
