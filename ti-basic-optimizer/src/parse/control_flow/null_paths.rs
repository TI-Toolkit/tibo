use crate::data::Program;
use crate::parse::{components, Parse};
use titokens::{Token, Tokens};

use std::collections::BTreeMap;

fn skip_delvars(line: &mut Tokens) -> Option<Token> {
    let mut token = line.next();

    while let Some(Token::TwoByte(0xBB, 0x54)) = token {
        components::DelVarTarget::parse(line.next().unwrap(), line);

        token = line.next()
    }

    token
}

#[derive(Debug)]
struct Branch {
    kind: BranchKind,
    idx: usize,
    delvar: bool,
}

#[derive(Debug)]
enum BranchKind {
    IfThen,
    Else { has_if_then: bool },
    SkippableLoop,
    UnskippableLoop,
}

/// - For every `If-Then`, locates the `Else` or `End` that the `If-Then` will take if the condition
/// is falsy.
/// - For every `Else`, locates the `End` that the `Else` will take if the condition on the `If-Then`
/// was truthy.
/// - For every `While` and `For(`, locates the `End` that the loop will jump to if the condition
/// fails immediately.
///
/// Returns a `BTreeMap` mapping the line of the source statement to the line *after* the
/// `End`/`Else` that was found.
fn find_null_paths(program: Program) -> BTreeMap<usize, usize> {
    let mut lines = program.lines.iter().enumerate().peekable();

    let mut output: BTreeMap<usize, usize> = BTreeMap::new();

    let mut stack = vec![];

    while let Some((idx, line)) = lines.next() {
        let mut tokens = Tokens::from_vec(line.tokens.clone(), program.version.clone());

        let (delvar, token) = if let Some(Token::TwoByte(0xBB, 0x54)) = tokens.peek() {
            (true, skip_delvars(&mut tokens))
        } else {
            (false, tokens.next())
        };

        match token {
            Some(Token::OneByte(0xD1 | 0xD3)) => {
                // While, For(
                stack.push(Branch {
                    kind: BranchKind::SkippableLoop,
                    idx,
                    delvar,
                });
            }

            Some(Token::OneByte(0xD2)) => {
                // Repeat
                stack.push(Branch {
                    kind: BranchKind::UnskippableLoop,
                    idx,
                    delvar,
                });
            }

            Some(Token::OneByte(0xCE)) => {
                // If
                if let Some(Token::OneByte(0xCF)) = lines.peek().and_then(|(_, x)| x.tokens.first())
                {
                    // Then
                    stack.push(Branch {
                        kind: BranchKind::IfThen,
                        idx,
                        delvar,
                    })
                }
            }

            Some(Token::OneByte(0xD0)) => {
                // Else
                let has_if_then = if let Some((
                    if_then_stack_idx,
                    Branch {
                        kind: BranchKind::IfThen,
                        idx: line_idx,
                        ..
                    },
                )) = stack
                    .iter()
                    .rposition(|x| !x.delvar)
                    .map(|idx| (idx, stack.get(idx).unwrap()))
                {
                    output.insert(*line_idx, idx + 1);
                    stack.remove(if_then_stack_idx);

                    true
                } else {
                    // *possible* runtime error; can't make any assumptions here unfortunately

                    false
                };

                stack.push(Branch {
                    kind: BranchKind::Else { has_if_then },
                    idx,
                    delvar: false,
                })
            }

            Some(Token::OneByte(0xD4)) => {
                // End

                while let Some(branch) = stack.pop() {
                    // is it possible for Branches earlier to find it
                    match branch {
                        Branch {
                            delvar: true,
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

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use std::ptr::null;

    use test_files::load_test_data;

    #[test]
    fn control_flow_puzzle() {
        let program: Program =
            load_test_data("/snippets/parsing/control-flow-shenanigans/puzzle-solution.txt").into();

        let null_paths = find_null_paths(program);

        assert_eq!(null_paths.keys().collect_vec(), vec![&0, &1, &5, &7]);
        assert_eq!(null_paths.values().collect_vec(), vec![&10, &5, &9, &9]);
    }

    #[test]
    fn with_delvar() {
        let program: Program =
            load_test_data("/snippets/parsing/control-flow-shenanigans/delvar-control-flow.txt")
                .into();

        let null_paths = find_null_paths(program);

        assert_eq!(*null_paths.keys().collect_vec(), vec![&0, &3, &5, &7]);
        assert_eq!(*null_paths.values().collect_vec(), vec![&9, &9, &9, &9]);
    }
}
