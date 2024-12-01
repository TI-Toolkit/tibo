//! # Label Name Optimization
//! Label names are one or two tokens long. This pass forces the most commonly used labels to be one
//! token long.
//!
//! As a micro-optimization, we choose letters before numbers because letters seem to be faster by a
//! handful of CC's (perhaps they're checked first?).

use crate::parse::statements::control_flow::Menu;
use crate::parse::{
    statements::{ControlFlow, LabelName, Statement},
    Program,
};
use itertools::Itertools;

const DICTIONARY: [u8; 37] = [
    // A-Z, theta
    0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50,
    0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B, //
    // 0-9
    0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39,
];

const LETTERS: usize = 27;
const NUMBERS: usize = 10;

/// Produce a more optimal [`LabelName`] given the "usage rank" of the label (0 is the rank of the
/// most used label, 1 is the rank of the second most-used label, etc).
/// Currently, the pattern works like this, though this is subject to change.
/// ```text
/// 1) A .. theta, 0 .. 9
/// 2) AA .. Atheta, BA .. Btheta, ..., thetaA .. thetatheta
/// 3) 0A .. 0theta, 1A .. 1theta, ..., 9A .. 9theta
/// 3) A0 .. theta0, A1 .. theta1, ..., A9 .. theta9
/// 4) 00 .. 09, 10 .. 19, ..., 90 .. 99
/// ```
fn label_name(mut rank: usize) -> LabelName {
    if rank < DICTIONARY.len() {
        return LabelName::new(DICTIONARY[rank], None);
    }
    rank -= DICTIONARY.len();

    if rank < LETTERS * LETTERS {
        return LabelName::new(DICTIONARY[rank / LETTERS], Some(DICTIONARY[rank % LETTERS]));
    }
    rank -= LETTERS * LETTERS;

    if rank < NUMBERS * LETTERS {
        return LabelName::new(
            DICTIONARY[LETTERS + (rank / LETTERS)],
            Some(DICTIONARY[rank % LETTERS]),
        );
    }
    rank -= NUMBERS * LETTERS;

    if rank < LETTERS * NUMBERS {
        return LabelName::new(
            DICTIONARY[rank % LETTERS],
            Some(DICTIONARY[LETTERS + (rank / LETTERS)]),
        );
    }
    rank -= LETTERS * NUMBERS;

    LabelName::new(
        DICTIONARY[LETTERS + (rank / NUMBERS)],
        Some(DICTIONARY[LETTERS + (rank % NUMBERS)]),
    )
}

#[allow(rustdoc::private_intra_doc_links)]
impl Program {
    /// This optimization has two steps:
    /// - Clear unused label declarations
    /// - Optimize the length of label names so that more commonly used labels have shorter names.
    ///
    /// See also: [`label_name`]
    pub fn optimize_label_names(&mut self) {
        let label_declarations = self.label_declarations();
        let label_usages = self.label_usages();

        for (line_idx, line) in self.lines.iter_mut().enumerate() {
            if let Statement::ControlFlow(ControlFlow::Lbl(decl_label)) = line {
                if !label_usages.contains_key(decl_label)
                    || label_declarations[decl_label] != line_idx
                {
                    *line = Statement::None;
                }
            }
        }

        // At this moment, every label has exactly 1 + len(usages) occurrences in the program. Now,
        // we sort and rank them.

        let mut usage_sorted = label_usages.iter().collect::<Vec<_>>();
        usage_sorted.sort_by(|&a, &b| b.1.len().cmp(&a.1.len()));

        for (rank, &(label, usages)) in usage_sorted.iter().enumerate() {
            let new_name = label_name(rank);

            if let Some(&declaration_line) = label_declarations.get(label) {
                if let Some(Statement::ControlFlow(ControlFlow::Lbl(decl_label))) =
                    &mut self.lines.get_mut(declaration_line)
                {
                    *decl_label = new_name;
                } else {
                    panic!("Incorrect output from label_declarations, please report this.");
                }
            } else {
                panic!("Label used without accompanying Lbl statement.");
            }

            for i in 0..usages.len() {
                let usage = usages[i];
                if i != 0 && usages[i - 1] == usages[i] {
                    continue;
                }

                match &mut self.lines.get_mut(usage) {
                    Some(Statement::ControlFlow(ControlFlow::Goto(usage_label))) => {
                        *usage_label = new_name;
                    }

                    Some(Statement::ControlFlow(ControlFlow::Menu(Menu {
                        option_labels, ..
                    }))) => {
                        for used_label in option_labels {
                            if label == used_label {
                                *used_label = new_name;
                            }
                        }
                    }

                    // nothing else can use labels
                    _ => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::label_name;
    use std::collections::{BTreeMap, BTreeSet};
    use test_files::{load_test_data, test_tokenizer};

    // This is a correctness guarantee, by the pigeonhole principle and the fact that constructing a
    // LabelName performs a check on the validity of the bytes being passed.
    #[test]
    fn label_names_distinct() {
        let mut used_names = BTreeSet::new();
        for i in 0..=(DICTIONARY.len() * DICTIONARY.len()) {
            let name = label_name(i).internal_id();

            assert!(!used_names.contains(&name));
            used_names.insert(name);
        }
    }

    #[test]
    fn unused_labels_eliminated() {
        let mut tokens = load_test_data("/snippets/optimize/control-flow/unused-label.txt");
        let tokenizer = test_tokenizer!();

        let mut program = Program::from_tokens(&mut tokens, &tokenizer);
        assert_eq!(program.label_declarations().len(), 2);
        program.optimize_label_names();
        assert_eq!(program.label_declarations().len(), 1);
    }

    #[test]
    fn label_renaming() {
        let mut tokens = load_test_data("/snippets/optimize/control-flow/label-name.txt");
        let tokenizer = test_tokenizer!();

        let mut program = Program::from_tokens(&mut tokens, &tokenizer);
        program.optimize_label_names();

        let mut expected = BTreeMap::new();
        expected.insert(label_name!('A'), vec![0, 0, 0]);
        expected.insert(label_name!('B'), vec![0, 2]);
        expected.insert(label_name!('C'), vec![0]);

        assert_eq!(program.label_usages(), expected);
    }
}
