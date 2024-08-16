use ariadne;
use titokens::tokenizer::TokenBoundaries;
use titokens::Token;

#[derive(Debug, Clone)]
pub struct LineReport {
    location: usize,
    message: String,
    suggestion: String,

    labels: Vec<(std::ops::Range<usize>, String)>,
}

impl LineReport {
    #[must_use]
    pub fn new(location: usize, message: &str, suggestion: &str) -> Self {
        LineReport {
            location,
            message: message.to_string(),
            suggestion: suggestion.to_string(),

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
        let report = ariadne::Report::build(
            ariadne::ReportKind::Error,
            (),
            boundaries.single(self.location).start,
        )
        .with_message(self.message)
        .with_labels(self.labels.iter().map(|(range, message)| {
            ariadne::Label::new(boundaries.range(range.start..range.end)).with_message(message)
        }))
        .with_help(self.suggestion)
        .finish();

        report
            .eprint(ariadne::Source::from(boundaries.to_string()))
            .unwrap();
    }
}
