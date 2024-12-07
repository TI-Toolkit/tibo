use std::collections::BTreeMap;

use crate::data::intervals::PartitionMap;
use crate::parse::statements::control_flow::START_LABEL;
use crate::parse::{
    statements::{control_flow::Menu, ControlFlow, LabelName, Statement},
    Program,
};

impl Program {
    /// Compute a mapping from label names to the index of the line where the label was defined.
    pub fn label_declarations(&self) -> BTreeMap<LabelName, usize> {
        let mut declarations = BTreeMap::new();

        for (idx, line) in self.lines.iter().enumerate().rev() {
            if let Statement::ControlFlow(ControlFlow::Lbl(name)) = line {
                declarations.insert(*name, idx);
            }
        }

        declarations
    }

    pub fn line_to_label_map(&self) -> PartitionMap<usize, LabelName> {
        let mut declarations = self
            .label_declarations()
            .into_iter()
            .map(|(a, b)| {
                (a, b + 1) // map to line after label definition
            })
            .collect::<Vec<_>>();

        declarations.sort_by_key(|entry| entry.1);

        let (mut names, mut begins): (Vec<LabelName>, Vec<usize>) =
            declarations.into_iter().unzip();
        names.insert(0, START_LABEL);
        begins.insert(0, 0);

        PartitionMap::new(begins, names)
    }

    /// Compute a mapping from label names to label usages (namely, `Goto `, `Menu(`)
    ///
    /// If a `Menu(` references the same label name more than once, the line will appear in the
    /// usages that many times.
    pub fn label_usages(&self) -> BTreeMap<LabelName, Vec<usize>> {
        let mut usages: BTreeMap<LabelName, Vec<usize>> = BTreeMap::new();

        for (idx, line) in self.lines.iter().enumerate() {
            match line {
                Statement::ControlFlow(ControlFlow::Goto(label)) => {
                    usages.entry(*label).or_default().push(idx);
                }

                Statement::ControlFlow(ControlFlow::Menu(Menu { option_labels, .. })) => {
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
        expected.insert(label_name!('0'), vec![1, 1, 4]);

        assert_eq!(test_program.label_usages(), expected)
    }

    #[test]
    fn label_declarations() {
        let test_program = program();

        let mut expected = BTreeMap::new();
        expected.insert(label_name!('R' 'E'), 0usize);
        expected.insert(label_name!('P' 'L'), 2);
        expected.insert(label_name!('0'), 5);

        assert_eq!(test_program.label_declarations(), expected)
    }

    #[test]
    fn label_maps() {
        let test_program = program();

        let map = test_program.line_to_label_map();

        assert_eq!(map.find(&0), Some(&START_LABEL));
        assert_eq!(map.find(&1), Some(&label_name!('R' 'E')));
        assert_eq!(map.find(&2), Some(&label_name!('R' 'E')));
        assert_eq!(map.find(&3), Some(&label_name!('P' 'L')));
        assert_eq!(map.find(&4), Some(&label_name!('P' 'L')));
        assert_eq!(map.find(&6), Some(&label_name!('0')));
    }
}
