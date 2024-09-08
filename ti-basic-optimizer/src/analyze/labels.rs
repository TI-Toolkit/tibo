use std::collections::BTreeMap;

use crate::parse::commands::{control_flow::Menu, ControlFlow};
use crate::parse::{
    commands::{Command, LabelName},
    Program,
};

impl Program {
    /// Compute a mapping from label names to the index of the line where the label was defined.
    pub fn label_declarations(&self) -> BTreeMap<LabelName, usize> {
        let mut declarations = BTreeMap::new();

        for (idx, line) in self.lines.iter().enumerate() {
            if let Command::ControlFlow(ControlFlow::Lbl(name)) = line {
                declarations.insert(*name, idx);
            }
        }

        declarations
    }

    /// Compute a mapping from label names to label usages (namely, `Goto `, `Menu(`)
    ///
    /// If a `Menu(` references the same label name more than once, the line will appear in the
    /// usages that many times.
    pub fn label_usages(&self) -> BTreeMap<LabelName, Vec<usize>> {
        let mut usages: BTreeMap<LabelName, Vec<usize>> = BTreeMap::new();

        for (idx, line) in self.lines.iter().enumerate() {
            match line {
                Command::ControlFlow(ControlFlow::Goto(label)) => {
                    usages.entry(*label).or_default().push(idx);
                }

                Command::ControlFlow(ControlFlow::Menu(Menu { option_labels, .. })) => {
                    for label in option_labels {
                        usages.entry(*label).or_default().push(idx);
                    }
                }

                _ => {}
            }
        }

        usages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::label_name;
    use test_files::{load_test_data, test_tokenizer};

    fn program() -> Program {
        let mut tokens = load_test_data("/snippets/analysis/labels.txt");
        let tokenizer = test_tokenizer!();

        Program::from_tokens(&mut tokens, &tokenizer)
    }

    #[test]
    fn label_usages() {
        let test_program = program();

        let mut expected = BTreeMap::new();
        expected.insert(label_name!('R' 'E'), vec![1]);
        expected.insert(label_name!('P' 'L'), vec![1]);
        expected.insert(label_name!('0'), vec![1, 1, 3]);

        assert_eq!(test_program.label_usages(), expected)
    }

    #[test]
    fn label_declarations() {
        let test_program = program();

        let mut expected = BTreeMap::new();
        expected.insert(label_name!('R' 'E'), 0usize);
        expected.insert(label_name!('P' 'L'), 2);
        expected.insert(label_name!('0'), 4);

        assert_eq!(test_program.label_declarations(), expected)
    }
}
