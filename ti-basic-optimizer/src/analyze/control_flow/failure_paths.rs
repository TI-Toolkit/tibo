//! Determine where conditionals will jump if their condition is false.
//!
//! This module provides [`Program::block_failure_paths`], [`Program::simple_failure_paths`], and [`Program::failure_paths`].

use crate::parse::{
    commands::{Command, ControlFlow, DelVarChain},
    Program,
};
use std::collections::BTreeMap;

/// Extremely simplified model of TI-BASIC control flow (only what is necessary for this code to work)
#[doc(hidden)]
#[derive(Debug)]
struct Branch {
    kind: BranchKind,
    idx: usize,
    is_delvar_valence: bool,
}

#[doc(hidden)]
#[derive(Debug)]
enum BranchKind {
    IfThen,
    Else { has_if_then: bool },
    SkippableLoop,
    UnskippableLoop,
}

impl Program {
    /// Determine where block-conditionals will jump if their condition is false:
    /// - For every `If-Then`, locates the `Else` or `End` that the `If-Then` will take if the condition
    ///   is falsy.
    /// - For every `Else`, locates the `End` that the `Else` will take if the condition on the `If-Then`
    ///   was truthy.
    /// - For every `While` and `For(`, locates the `End` that the loop will jump to if the condition
    ///   fails immediately.
    ///
    /// Returns a [`BTreeMap`] mapping the line of the source statement to the line *after* the
    /// `End`/`Else` that was found.
    ///
    /// Most of the logic here is explored in <https://www.cemetech.net/forum/viewtopic.php?p=307835>
    /// and <https://www.cemetech.net/forum/viewtopic.php?p=307861>
    pub fn block_failure_paths(&self) -> BTreeMap<usize, usize> {
        let mut lines = self.lines.iter().enumerate().peekable();
        let mut output: BTreeMap<usize, usize> = BTreeMap::new();

        let mut stack = vec![];

        // rustfmt butchers this and makes it much harder to read imo
        // todo: consider manually reformatting and telling rustfmt to ignore this block.
        while let Some((idx, mut command)) = lines.next() {
            let is_delvar_valence = if let Command::DelVarChain(DelVarChain {
                valence: Some(valence_command),
                ..
            }) = command
            {
                command = valence_command;
                true
            } else {
                false
            };

            if let Command::ControlFlow(cf) = command {
                match cf {
                    ControlFlow::While(_) | ControlFlow::For(_) => stack.push(Branch {
                        kind: BranchKind::SkippableLoop,
                        idx,
                        is_delvar_valence,
                    }),

                    ControlFlow::Repeat(_) => {
                        stack.push(Branch {
                            kind: BranchKind::UnskippableLoop,
                            idx,
                            is_delvar_valence,
                        });
                    }

                    ControlFlow::If(_) => match lines.peek() {
                        Some((_, Command::ControlFlow(ControlFlow::Then))) => stack.push(Branch {
                            kind: BranchKind::IfThen,
                            idx,
                            is_delvar_valence,
                        }),
                        Some(_) => {}
                        None => panic!("Expected If statement body"), // todo: make an error?
                    },

                    ControlFlow::Else => {
                        let has_if_then = if let Some((
                            if_then_stack_idx,
                            Branch {
                                kind: BranchKind::IfThen,
                                idx: line_idx,
                                ..
                            },
                        )) = stack
                            .iter()
                            .rposition(|x| !x.is_delvar_valence)
                            .map(|idx| (idx, stack.get(idx).unwrap()))
                        {
                            if !is_delvar_valence {
                                output.insert(*line_idx, idx + 1);
                                stack.remove(if_then_stack_idx);
                            }

                            true
                        } else {
                            // *possible* runtime error; can't make any assumptions here unfortunately

                            false
                        };

                        stack.push(Branch {
                            kind: BranchKind::Else { has_if_then },
                            idx,
                            is_delvar_valence,
                        })
                    }

                    ControlFlow::End => {
                        while let Some(branch) = stack.pop() {
                            match branch {
                                Branch {
                                    is_delvar_valence: true,
                                    idx: line_idx,
                                    ..
                                }
                                | Branch {
                                    kind: BranchKind::Else { has_if_then: false },
                                    idx: line_idx,
                                    ..
                                } => {
                                    output.insert(line_idx, idx + 1);
                                }

                                Branch {
                                    kind: _,
                                    idx: line_idx,
                                    ..
                                } => {
                                    output.insert(line_idx, idx + 1);

                                    break;
                                }
                            }
                        }
                    }

                    _ => {}
                }
            }
        }

        output
    }

    /// Conditionals like `Is>(`, `Ds<(`, and `If` without a `Then` skip a single line. 
    /// 
    /// Returns a [`BTreeMap`] mapping the line of the source statement to the line after
    /// the skipped line.
    pub fn simple_failure_paths(&self) -> BTreeMap<usize, usize> {
        let max_line_idx = self.lines.len();

        let mut lines = self.lines.iter().enumerate().peekable();
        let mut output: BTreeMap<usize, usize> = BTreeMap::new();

        while let Some((idx, mut command)) = lines.next() {
            if let Command::DelVarChain(DelVarChain {
                valence: Some(valence_cmd),
                ..
            }) = command
            {
                command = valence_cmd;
            }

            if let Command::ControlFlow(cf) = command {
                match cf {
                    ControlFlow::If(_) => match lines.peek() {
                        Some((_, Command::ControlFlow(ControlFlow::Then))) => {}
                        Some(_) => {
                            output.insert(idx, idx + 2);

                            if idx + 2 > max_line_idx {
                                // todo: make an error?
                                panic!("If statement has nowhere to jump to when false")
                            }
                        }
                        None => panic!("Expected If statement body"), // todo: make an error?
                    },
                    ControlFlow::IsGt(_) | ControlFlow::DsLt(_) => {
                        output.insert(idx, idx + 2);
                        if idx + 2 > max_line_idx {
                            panic!("Is>/Ds< statement has nowhere to jump to when false")
                        }
                    }

                    _ => {}
                }
            }
        }

        output
    }

    /// Union of [`Program::simple_failure_paths`] and [`Program::block_failure_paths`].
    pub fn failure_paths(&self) -> BTreeMap<usize, usize> {
        let mut all = self.simple_failure_paths();
        for (source, dest) in self.block_failure_paths() {
            assert!(!all.contains_key(&source));
            all.insert(source, dest);
        }

        all
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    use test_files::{load_test_data, test_tokenizer};

    #[test]
    fn control_flow_puzzle() {
        let mut tokens =
            load_test_data("/snippets/parsing/control-flow-shenanigans/puzzle-solution.txt");
        let program = Program::from_tokens(&mut tokens, &test_tokenizer!());

        let failure_paths = program.failure_paths();

        assert_eq!(
            failure_paths.keys().collect::<Vec<_>>(),
            vec![&0, &1, &5, &7]
        );
        assert_eq!(failure_paths.values().collect_vec(), vec![&10, &5, &9, &9]);
    }

    #[test]
    fn with_delvar() {
        let mut tokens =
            load_test_data("/snippets/parsing/control-flow-shenanigans/delvar-control-flow.txt");
        let program = Program::from_tokens(&mut tokens, &test_tokenizer!());

        let failure_paths = program.failure_paths();

        assert_eq!(
            *failure_paths.keys().collect_vec(),
            vec![&0, &3, &5, &7, &10, &12, &14, &16]
        );
        assert_eq!(
            *failure_paths.values().collect_vec(),
            vec![&9, &9, &9, &9, &12, &14, &19, &19]
        );
    }
}
