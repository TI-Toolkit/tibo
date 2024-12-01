use std::{
    collections::{BTreeMap, BTreeSet},
    os::linux::raw::stat,
};

use tifloats::{tifloat, Float};
use titokens::Token;

use crate::{
    data::{
        graph_like::{Digraph, NodeIndex},
        intervals::{IntervalTree, PartitionMap},
    },
    error_reporting::LineReport,
    parse::{
        components::{BinOp, FunctionCall, Operand, Operator},
        expression::Expression,
        statements::{
            control_flow::{ForLoop, IsDs, Menu, START_LABEL},
            ControlFlow, LabelName, Statement,
        },
        Program,
    },
};

#[cfg(feature = "debug-tools")]
mod graphviz;

const ONE: Expression = Expression::Operand(Operand::NumericLiteral(tifloat!(
    0x0010000000000000 * 10 ^ 0
)));

#[doc(alias = "cfg")]
pub struct ControlFlowGraph {
    graph: Digraph<BasicBlock>,
}

impl From<Digraph<BasicBlock>> for ControlFlowGraph {
    fn from(value: Digraph<BasicBlock>) -> Self {
        Self { graph: value }
    }
}

/// Simply put, Flow answers the question of "how do we decide which out-edge to take?"
#[derive(Clone, Debug, Default)]
pub enum Flow {
    #[default]
    Unknown,
    Jump,
    Branch(Expression),
    ForBranch(ForLoop),
    Menu(Menu),
    Goto(LabelName),
    Meta,

    ProgramEnd {
        ret: bool,
    },
    SubgraphEnd,
}

impl Flow {
    pub fn is_unknown(&self) -> bool {
        matches!(self, Flow::Unknown)
    }
}

#[derive(Clone, Debug)]
pub struct BasicBlock {
    pub lines: Vec<Statement>,
    pub flow: Flow,

    pub starting_line: usize,
}

impl BasicBlock {
    pub fn new(starting_line: usize) -> Self {
        Self {
            lines: vec![],
            flow: Flow::Unknown,

            starting_line,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn push_line(&mut self, line: Statement) {
        self.lines.push(line);
    }
}

pub enum Segment {
    Literal {
        statement: ControlFlow,
        failure_connection: Option<LabelName>,
    },
    Blocks(ControlFlowGraph),
}

pub struct LabelFragment {
    pub name: LabelName,
    pub data: Vec<Segment>,

    eof_abuse: usize,
}

impl LabelFragment {
    pub fn new(name: LabelName) -> Self {
        Self {
            name,
            data: vec![],

            eof_abuse: 0,
        }
    }

    pub fn push_cfg(&mut self, cfg: ControlFlowGraph) {
        self.data.push(Segment::Blocks(cfg));
    }

    pub fn push_literal(&mut self, literal: ControlFlow) {
        self.data.push(Segment::Literal {
            statement: literal,
            failure_connection: None,
        });
    }
}

/// Struct to hold several different useful precomputations for constructing control flow
#[derive(Debug)]
pub(super) struct ControlFlowLookup {
    /// TIBO treats conditionals that define a lexical block and contain a Goto, Lbl, or Menu differently.
    /// These statements are not part of control flow graphs but are optimized and handled separately.
    /// Unlike in a fully structured program, specific changes to these statements (eg. converting a
    /// `While` to a `For(`) could lead to invalid or wildly difficult-to-manage effects.
    ///
    /// This function scans the program and searches for these conditionals. If a conditional contains a
    /// lexical block and a Goto, Lbl, or Menu appears within that lexical block or one nested within it,
    /// the conditional's line number appears in this set. Furthermore, because the structured
    /// subprograms must have balanced Ends, any unbalanced Ends or End possibly corresponding to the
    /// conditionals in this set also appear in this set.
    pub literals: BTreeSet<usize>,
    /// Line numbers of unclosed control flow statement which use the end of the program to terminate the
    /// program.
    pub eof_abusers: BTreeSet<usize>,
    /// See [`Program::block_failure_paths`]
    pub block_failure_paths: BTreeMap<usize, usize>,

    pub lexical_blocks: IntervalTree<usize>,

    pub failure_connections: BTreeMap<usize, (LabelName, usize)>,

    pub labels: PartitionMap<usize, LabelName>,
    pub label_usages: BTreeMap<LabelName, Vec<usize>>,
    pub label_declarations: BTreeMap<LabelName, usize>,

    pub line_count: usize,
}

impl Program {
    /// Assumes [`Program::optimize_label_names`] has been run to remove duplicate labels.
    fn control_flow_lookup(&self) -> Result<Box<ControlFlowLookup>, LineReport> {
        let (block_failure_paths, eof_abusers) = self.block_failure_paths();

        let labels = self.line_to_label_map();
        let label_usages = self.label_usages();
        let label_declarations = self.label_declarations();

        let mut lexical_blocks = vec![];
        let mut failure_connections = BTreeMap::new();
        for (conditional, destination) in block_failure_paths.iter() {
            if !labels.in_same_range(conditional, &(destination - 1)) {
                failure_connections.insert(
                    *conditional,
                    (*labels.find(&(destination - 1)).unwrap(), destination - 1),
                );
            }

            lexical_blocks.push(*conditional..*destination);
        }
        let lexical_blocks = IntervalTree::new(lexical_blocks);

        let mut literals = BTreeSet::new();
        for usages in label_usages.values() {
            for usage in usages {
                let query = lexical_blocks.stab(*usage);
                for range in query {
                    literals.insert(range.start);
                    literals.insert(range.end - 1);
                }
            }
        }

        for usage in label_declarations
            .values()
            .chain(label_usages.values().flatten())
        {
            let query: Vec<std::ops::Range<usize>> = lexical_blocks.stab(*usage);
            for range in query {
                literals.insert(range.start);
                literals.insert(range.end);
            }
        }

        for (statement, failure_path) in block_failure_paths.iter() {
            if matches!(
                self.lines[*statement],
                Statement::ControlFlow(ControlFlow::IfThen(_))
            ) && matches!(
                self.lines[*failure_path - 1],
                Statement::ControlFlow(ControlFlow::Else)
            ) && literals.contains(&(failure_path - 1))
            {
                literals.insert(*statement);
            }
        }

        Ok(Box::new(ControlFlowLookup {
            literals,
            eof_abusers,
            block_failure_paths,
            lexical_blocks,
            failure_connections,
            labels,
            label_usages,
            label_declarations,
            line_count: self.lines.len(),
        }))
    }

    #[cfg(feature = "debug-tools")]
    pub fn to_cfg(mut self) -> Result<Vec<LabelFragment>, LineReport> {
        // Strictly reduces number of labels -> less work
        self.optimize_label_names();

        let cfl = self.control_flow_lookup()?;
        let mut parser = ControlFlowParser::new(cfl);

        for (idx, stmt) in self.lines.into_iter().enumerate() {
            parser.next_line(idx, stmt)?
        }

        parser.finish();

        todo!()
    }
}

struct ControlFlowParser {
    cfl: Box<ControlFlowLookup>,

    active_simple_conds: usize,

    cf_stack: Vec<(usize, Flow)>,
    fragments: Vec<LabelFragment>,

    cur_fragment: LabelFragment,
    cur_subgraph: Digraph<BasicBlock>,
    cur_block: BasicBlock,

    block_map: BTreeMap<usize, NodeIndex>,

    /// Pairs of (`source block start line`,`destination block start line`)
    ///
    /// Edges are added in reverse order of being pushed (i.e. push success edges first, then failure edges).
    waiting_edges: Vec<(usize, usize)>,
}

impl ControlFlowParser {
    pub fn new(cfl: Box<ControlFlowLookup>) -> Self {
        Self {
            cfl,

            active_simple_conds: 0,

            cf_stack: vec![],
            fragments: vec![],

            cur_fragment: LabelFragment::new(START_LABEL),
            cur_subgraph: Digraph::new(),
            cur_block: BasicBlock::new(0),

            block_map: BTreeMap::new(),

            waiting_edges: vec![],
        }
    }

    pub fn next_line(&mut self, line_index: usize, statement: Statement) -> Result<(), LineReport> {
        if self.active_simple_conds > 0 {
            self.active_simple_conds -= 1;

            if self.active_simple_conds != 1 && self.cur_block.starting_line != line_index {
                let start = self.finish_block(Flow::Jump, line_index);
                self.add_edge(start, line_index);
            }
        }

        match statement {
            Statement::None => Ok(()),
            Statement::ControlFlow(cf) => self.handle_control_flow(line_index, cf),
            Statement::DelVarChain(_) if statement.is_control_flow() => Err(LineReport::new(
                line_index,
                "Unsupported DelVar'ed Control Flow.",
                Some("This may be supported in later versions of the optimizer."),
            )),

            stmt => {
                self.cur_block.push_line(stmt);

                Ok(())
            }
        }
    }

    fn handle_control_flow(
        &mut self,
        line_index: usize,
        cf: ControlFlow,
    ) -> Result<(), LineReport> {
        if self.cfl.literals.contains(&line_index) {
            self.finish_subgraph(line_index + 1);
            self.cur_fragment.push_literal(cf);

            return Ok(());
        }

        match cf {
            ControlFlow::Lbl(label_name) => {
                self.finish_fragment(label_name, line_index + 1);
            }

            ControlFlow::Goto(label_name) => {
                self.finish_block(Flow::Goto(label_name), line_index + 1);
            }

            ControlFlow::Menu(menu) => {
                self.finish_block(Flow::Menu(menu), line_index + 1);
            }

            ControlFlow::If(cond) => self.handle_simple_conditional(line_index, cond),

            ControlFlow::IsGt(isds) => self.handle_isds(line_index, isds, true),
            ControlFlow::DsLt(isds) => self.handle_isds(line_index, isds, false),

            ControlFlow::Repeat(condition) => {
                self.add_edge(self.cur_block.starting_line, line_index + 1);
                self.finish_block(Flow::Jump, line_index + 1);

                self.cf_stack.push((
                    line_index + 1,
                    Flow::Branch(Expression::Operator(Operator::FunctionCall(FunctionCall {
                        kind: Token::OneByte(0xB8), // not(
                        arguments: vec![condition],
                    }))),
                ));
            }

            ControlFlow::While(condition) => {
                let failure_path: usize = *self.cfl.block_failure_paths.get(&line_index).unwrap();

                self.add_edge(self.cur_block.starting_line, line_index + 1);
                self.add_edge(self.cur_block.starting_line, failure_path);
                self.finish_block(Flow::Branch(condition.clone()), line_index + 1);

                self.cf_stack
                    .push((line_index + 1, Flow::Branch(condition)));
            }

            ControlFlow::For(forloop) => self.handle_for_loop(line_index, forloop)?,

            ControlFlow::IfThen(cond) => {
                self.add_edge(self.cur_block.starting_line, line_index + 1);

                let failure_path: usize = *self.cfl.block_failure_paths.get(&line_index).unwrap();
                self.add_edge(self.cur_block.starting_line, failure_path);
                self.finish_block(Flow::Branch(cond), line_index + 1);

                self.cf_stack.push((usize::MAX, Flow::Jump));
            }

            ControlFlow::Else => {
                let failure_path: usize = *self.cfl.block_failure_paths.get(&line_index).unwrap();
                self.add_edge(self.cur_block.starting_line, failure_path);

                if !matches!(self.cf_stack.last(), Some((_, Flow::Jump))) {
                    self.cf_stack.push((usize::MAX, Flow::Jump));
                }

                self.finish_block(Flow::Branch(ONE), line_index + 1);
            }

            ControlFlow::End => {
                if self.active_simple_conds != 0 {
                    Err(LineReport::new(
                        line_index,
                        "Unsupported control flow: If-End",
                        None,
                    ))?
                }

                if let Some((back_edge, flow)) = self.cf_stack.pop() {
                    if !matches!(flow, Flow::Jump) {
                        self.add_edge(self.cur_block.starting_line, back_edge);
                    }

                    self.add_edge(self.cur_block.starting_line, line_index + 1);

                    self.finish_block(flow, line_index + 1);
                } else {
                    self.finish_subgraph(line_index + 1);
                    self.cur_fragment.push_literal(cf);
                }
            }

            ControlFlow::Return => {
                self.finish_block(Flow::ProgramEnd { ret: true }, line_index + 1);
            }

            ControlFlow::Stop => {
                self.finish_block(Flow::ProgramEnd { ret: false }, line_index + 1);
            }

            _ => {}
        }

        Ok(())
    }

    fn handle_isds(&mut self, line_index: usize, isds: IsDs, increment: bool) {
        let mutator = if increment {
            Token::OneByte(0x70) // +
        } else {
            Token::OneByte(0x71) // -
        };

        let comparator = if increment {
            Token::OneByte(0x6D)
        } else {
            Token::OneByte(0x6E)
        };

        self.cur_block
            .push_line(Statement::Fiction(Box::new(Statement::Store(
                Expression::Operator(Operator::Binary(BinOp {
                    kind: mutator,
                    left: isds.variable.into(),
                    right: Box::new(ONE),
                })),
                isds.variable.into(),
            ))));

        self.handle_simple_conditional(
            line_index,
            Expression::Operator(Operator::Binary(BinOp {
                kind: comparator,
                left: isds.variable.into(),
                right: Box::new(isds.condition),
            })),
        );
    }

    fn handle_simple_conditional(&mut self, line_index: usize, condition: Expression) {
        self.active_simple_conds = 2;

        self.add_edge(self.cur_block.starting_line, line_index + 1);
        self.add_edge(self.cur_block.starting_line, line_index + 2);

        self.finish_block(Flow::Branch(condition), line_index + 1);
    }

    fn handle_for_loop(&mut self, line_index: usize, forloop: ForLoop) -> Result<(), LineReport> {
        if let Expression::Operand(Operand::NumericVarName(nvn)) = forloop.iterator {
            let failure_path: usize = *self.cfl.block_failure_paths.get(&line_index).unwrap();

            self.add_edge(self.cur_block.starting_line, line_index + 1);
            self.add_edge(self.cur_block.starting_line, failure_path);
            self.cur_block
                .push_line(Statement::Fiction(Box::new(Statement::Store(
                    forloop.start.clone(),
                    nvn.into(),
                ))));

            let difference = Box::new(Expression::Operator(Operator::Binary(BinOp {
                kind: Token::OneByte(0x71), // -
                left: Box::new(forloop.end.clone()),
                right: Box::new(forloop.start.clone()),
            })));

            let condition = if let Some(step) = forloop.step.clone() {
                Box::new(Expression::Operator(Operator::Binary(BinOp {
                    kind: Token::OneByte(0x83), // /
                    left: difference,
                    right: Box::new(step),
                })))
            } else {
                difference
            };

            self.finish_block(
                Flow::Branch(Expression::Operator(Operator::Binary(BinOp {
                    kind: Token::OneByte(0x6C), // >
                    left: condition,
                    right: tifloat!(0 * 10 ^ 0).into(),
                }))),
                line_index + 1,
            );

            self.cf_stack
                .push((line_index + 1, Flow::ForBranch(forloop)));

            Ok(())
        } else {
            Err(LineReport::new(
                line_index,
                "Unsupported loop variable.",
                None,
            ))
        }
    }

    pub fn add_edge(&mut self, source: usize, dest: usize) {
        self.waiting_edges.push((source, dest));
    }

    pub fn finish_block(&mut self, flow: Flow, next_line: usize) -> usize {
        self.cur_block.flow = flow;

        let starting_line = self.cur_block.starting_line;
        let node_index = self.cur_subgraph.insert_node(std::mem::replace(
            &mut self.cur_block,
            BasicBlock::new(next_line),
        ));

        self.block_map.insert(starting_line, node_index);

        starting_line
    }

    pub fn finish_subgraph(&mut self, next_line: usize) {
        self.finish_block(Flow::SubgraphEnd, next_line);

        while let Some((source, dest)) = self.waiting_edges.pop() {
            let source_block = *self
                .block_map
                .get(&source)
                .expect("Internal error during CFG creation! (1)");
            let dest_block = *self
                .block_map
                .get(&dest)
                .expect("Internal error during CFG creation! (2)");

            self.cur_subgraph.insert_arc(source_block, dest_block);
        }

        self.cur_fragment
            .push_cfg(std::mem::replace(&mut self.cur_subgraph, Digraph::new()).into());
    }

    pub fn finish_fragment(&mut self, next_label_name: LabelName, next_line: usize) {
        self.finish_subgraph(next_line);

        self.fragments.push(std::mem::replace(
            &mut self.cur_fragment,
            LabelFragment::new(next_label_name),
        ));
    }

    #[cfg(feature = "debug-tools")]
    pub fn finish(mut self) {
        self.handle_control_flow(self.cfl.line_count, ControlFlow::Return)
            .unwrap();

        self.finish_fragment(START_LABEL, self.cfl.line_count);
        use crate::data::graphviz::Graphviz;
        self.fragments.dbg();
    }
}
