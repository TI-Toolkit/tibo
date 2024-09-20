use std::ops::Range;
use titokens::tokenizer::TokenBoundaries;

macro_rules! next_or_err {
    ($tokens: ident) => {
        $tokens.next().ok_or_else(|| {
            (crate::error_reporting::TokenReport::new(
                $tokens.current_position() - 2,
                "Unexpected end of input.",
                None,
            )
            .with_label($tokens.current_position() - 2, "here"))
        })
    };

    ($tokens: ident, $message: literal) => {
        $tokens.next().ok_or_else(|| {
            (crate::error_reporting::TokenReport::new(
                $tokens.current_position() - 2,
                $message,
                None,
            )
            .with_label($tokens.current_position() - 2, "here"))
        })
    };
}

macro_rules! expect_tok {
    ($tokens: ident, $token: pat, $token_name: literal) => {
        crate::error_reporting::next_or_err!($tokens).and_then(|tok| {
            if !matches!(tok, $token) {
                Err(crate::error_reporting::TokenReport::new(
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
                Err(crate::error_reporting::TokenReport::new(
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
            crate::error_reporting::TokenReport::new(
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
            crate::error_reporting::TokenReport::new(
                $tokens.current_position() - 1,
                concat!("Expected to find ", $expected_kind, "."),
                None,
            )
            .with_label($tokens.current_position() - 1, concat!($help))
        })
    };

    ($option: expr, $tokens: ident, $ofs: expr, $expected_kind: literal, $help: literal) => {
        $option.ok_or_else(|| {
            crate::error_reporting::TokenReport::new(
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
    fn string_indices(&self, token_boundaries: &TokenBoundaries) -> Range<usize> {
        match self {
            LabelKind::Single(tok_idx) => token_boundaries.single(*tok_idx),
            LabelKind::Span(tok_range) => token_boundaries.range(tok_range.clone()),
        }
    }
}

pub trait Report: Sized {
    /// Format and print this error to stderr, using the provided [`TokenBoundaries`] to translate
    /// the tokens. This does
    ///
    /// The `ariadne` crate seems to choke on Unicode input; tokenize without Unicode.
    fn report(self, boundaries: TokenBoundaries) {
        self.translate(&boundaries)
            .eprint(ariadne::Source::from(boundaries.to_string()))
            .unwrap();
    }

    fn translate<'a>(self, boundaries: &TokenBoundaries) -> ariadne::Report<'a>;
}

/// `TokenReport` is used to report errors at the token level.
///
/// Token indices are usually obtained by calling [`Tokens::current_position`](titokens::Tokens::current_position).
#[derive(Debug, Clone)]
#[must_use]
pub struct TokenReport {
    location: usize,
    message: String,
    suggestion: Option<String>,
    code: Option<u16>,

    labels: Vec<(LabelKind, String)>,
}

impl TokenReport {
    /// New error at the provided token index.
    ///
    /// Token indices are usually obtained by calling [`Tokens::current_position`](titokens::Tokens::current_position).
    pub fn new(location: usize, message: &str, suggestion: Option<&str>) -> Self {
        TokenReport {
            location,
            message: message.to_string(),
            suggestion: suggestion.map(|x| x.to_string()),
            code: None,

            labels: vec![],
        }
    }

    /// Add a label at the provided range of token indices.
    ///
    /// Token indices are usually obtained by calling [`Tokens::current_position`](titokens::Tokens::current_position).
    pub fn with_span_label(mut self, location: Range<usize>, message: &str) -> Self {
        self.labels
            .push((LabelKind::Span(location), message.to_string()));

        self
    }

    /// Add a label at the provided token index.
    ///
    /// Token indices are usually obtained by calling [`Tokens::current_position`](titokens::Tokens::current_position).
    pub fn with_label(mut self, location: usize, message: &str) -> Self {
        self.labels
            .push((LabelKind::Single(location), message.to_string()));

        self
    }

    /// Provide an error code for this error.
    pub fn with_code(mut self, error_code: u16) -> Self {
        self.code = Some(error_code);

        self
    }
}

impl Report for TokenReport {
    fn translate<'a>(self, boundaries: &TokenBoundaries) -> ariadne::Report<'a> {
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
                ariadne::Label::new(label_kind.string_indices(boundaries)).with_message(message)
            }))
        }

        if let Some(help) = self.suggestion {
            builder = builder.with_help(help);
        }

        if let Some(code) = self.code {
            builder = builder.with_code(code);
        }

        builder.finish()
    }
}

/// `LineReport` is used to report errors which occur on a single line. The entire line is marked as an error.
#[derive(Clone, Debug)]
#[must_use]
pub struct LineReport {
    location: usize,
    message: String,
    suggestion: Option<String>,
}

impl LineReport {
    /// Construct a new [`LineReport`] at the provided line.
    pub fn new(location: usize, message: &str, suggestion: Option<&str>) -> Self {
        LineReport {
            location,
            message: message.to_string(),
            suggestion: suggestion.map(|x| x.to_string()),
        }
    }
}

impl Report for LineReport {
    fn translate<'a>(self, boundaries: &TokenBoundaries) -> ariadne::Report<'a> {
        let mut line_start_idx = None;
        let mut line_end_idx = None;
        // this is pretty expensive but we only have to do it once so it's not really worth doing anything smarter
        let mut line_idx = 0;
        for token_idx in 0..boundaries.len() {
            if boundaries.str_at_single(token_idx) == "\n" {
                line_idx += 1;
                if line_idx == self.location {
                    line_start_idx = Some(boundaries.single(token_idx).end);
                } else if line_idx == self.location + 1 {
                    line_end_idx = Some(boundaries.single(token_idx).start);
                }
            }
        }

        if line_start_idx.is_none() {
            // we *are* in the error reporting code. perhaps we could give a custom report? Problem: we don't know where to say the report
            // is supposed to be, and misleading the user is worse than giving something vague.
            panic!(
                "Internal Error: Invalid line number ({0}, max {line_idx}) for error report.",
                self.location
            );
        }

        if line_end_idx.is_none() {
            line_end_idx = Some(boundaries.single(boundaries.len() - 1).end);
        }

        let range = line_start_idx.unwrap()..line_end_idx.unwrap();

        let mut builder = ariadne::Report::build(ariadne::ReportKind::Error, (), range.start)
            .with_label(ariadne::Label::new(range).with_message(self.message));

        if let Some(suggestion) = self.suggestion {
            builder = builder.with_help(suggestion);
        }

        builder.finish()
    }
}
