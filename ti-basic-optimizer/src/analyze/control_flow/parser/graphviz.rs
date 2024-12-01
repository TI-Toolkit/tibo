use dot_writer::{Attributes, DotWriter, Style};
use equidistributed_colors::EquiColor;
use itertools::Itertools;
use test_files::test_version;
use titokens::Tokenizer;

use crate::{
    analyze::control_flow::parser::Flow,
    data::graphviz::{escape, Graphviz, Visualize},
    parse::{statements::control_flow::START_LABEL, Stringify},
};

use super::{BasicBlock, ControlFlowGraph, LabelFragment, Segment};

#[derive(Clone)]
pub struct BasicBlockVisSettings<'a> {
    pub tokenizer: Option<&'a Tokenizer>,
    pub color: String,
    pub border: Style,

    pub name: String,
}

impl<'a> Visualize<BasicBlockVisSettings<'a>> for BasicBlock {
    fn visualize(&self, context: &mut dot_writer::Scope, config: BasicBlockVisSettings) {
        let mut node = context.node_named(config.name);

        if config.color != "black" {
            node.set("color", &config.color, true);
        }

        node.set_style(config.border);

        node.set_font("Courier");
        let shape = match self.flow {
            Flow::ProgramEnd { ret: true } => "trapezium",
            Flow::ProgramEnd { ret: false } => "invtrapezium",

            Flow::SubgraphEnd => "note",

            Flow::Goto(_) => "box3d",

            _ => "rectangle",
        };

        node.set("shape", shape, false);

        let content = self
            .lines
            .iter()
            .map(|statement| statement.stringify(config.tokenizer))
            .join("\n");

        if let Flow::Goto(label_name) = self.flow {
            node.set("xlabel", &label_name.stringify(config.tokenizer), false);
        }

        node.set_label(&escape(&content).replace(
            "\\n",
            if shape.contains("trapezium") {
                "\\n"
            } else {
                "\\l"
            },
        ));
    }
}

#[derive(Clone)]
pub struct CFGVisSettings<'a> {
    pub tokenizer: Option<&'a Tokenizer>,
    pub namespace: String,
}

impl<'a> Visualize<CFGVisSettings<'a>> for ControlFlowGraph {
    fn visualize(&self, context: &mut dot_writer::Scope, config: CFGVisSettings) {
        let equi_color = EquiColor::new(0.5, 0.5);

        let colors = equi_color
            .take(self.graph.size())
            .map(|color| format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b))
            .collect::<Vec<String>>();

        self.graph.nodes().for_each(|(index, block)| {
            let block_color = colors[index.id()].clone();

            let config = BasicBlockVisSettings {
                tokenizer: config.tokenizer,
                color: block_color,
                border: Style::Solid,

                name: config.namespace.clone() + &index.to_string(),
            };

            block.visualize(context, config);
        });

        self.graph.arcs().for_each(|(source, sink)| {
            let mut edge = context
                .edge(
                    config.namespace.clone() + &source.to_string(),
                    config.namespace.clone() + &sink.to_string(),
                )
                .attributes();

            if let Some(color) = colors.get(source.id()) {
                edge.set("color", color, true);
            }

            let annotation = match &self.graph.node(source).flow {
                Flow::Jump => "",
                Flow::Branch(expression) => {
                    if self.graph.out_arcs(source)[0] == sink {
                        "else"
                    } else {
                        &expression.stringify(config.tokenizer)
                    }
                }
                Flow::ForBranch(forloop) => {
                    if self.graph.out_arcs(source)[0] == sink {
                        "else"
                    } else {
                        &format!(
                            "{} in [{},{}] by {}",
                            forloop.iterator.stringify(config.tokenizer),
                            forloop.start.stringify(config.tokenizer),
                            forloop.end.stringify(config.tokenizer),
                            forloop.step().stringify(config.tokenizer)
                        )
                    }
                }
                Flow::Meta => "META",
                Flow::ProgramEnd { .. } => "PROGRAM END",
                Flow::Goto(label) => &label.stringify(config.tokenizer),

                _ => "?",
            };

            edge.set_label(&escape(annotation)).set_font("Courier");
        });
    }
}

impl<'a> Visualize<CFGVisSettings<'a>> for Segment {
    fn visualize(&self, context: &mut dot_writer::Scope, config: CFGVisSettings) {
        match self {
            Segment::Literal {
                statement: control_flow,
                ..
            } => {
                let mut node = context.node_named(config.namespace.clone() + "_HANDLE");
                node.set_label(&escape(&control_flow.stringify(config.tokenizer)))
                    .set_font("Courier");

                node.set("shape", "\"component\"", false);
            }

            Segment::Blocks(cfg) => {
                let _handle = context
                    .node_named(config.namespace.clone() + "_HANDLE")
                    .set("shape", "\"point\"", false)
                    .set("style", "\"invis\"", false);

                cfg.visualize(context, config);
            }
        }
    }
}

impl<'a> Visualize<CFGVisSettings<'a>> for LabelFragment {
    fn visualize(&self, context: &mut dot_writer::Scope, config: CFGVisSettings) {
        let namespace = format!("c{}", self.name.internal_id());

        let mut last_namespace: String = "".into();
        for (idx, segment) in self.data.iter().enumerate() {
            let namespace = namespace.clone() + "_" + &idx.to_string();

            if idx != 0 {
                context
                    .edge(
                        last_namespace.to_string() + "_HANDLE",
                        namespace.clone() + "_HANDLE",
                    )
                    .attributes()
                    .set("lhead", &("cluster".to_string() + &namespace), true)
                    .set("ltail", &("cluster".to_string() + &last_namespace), true);
            }

            let mut inner_context = context.subgraph_named("cluster".to_string() + &namespace);

            if idx == 0 && self.name != START_LABEL {
                inner_context
                    .graph_attributes()
                    .set_label(&self.name.stringify(config.tokenizer));
            }

            segment.visualize(
                &mut inner_context,
                CFGVisSettings {
                    tokenizer: config.tokenizer,
                    namespace: namespace.clone(),
                },
            );

            last_namespace = namespace;
        }
    }
}

impl Graphviz for Vec<LabelFragment> {
    fn graphviz(&self, writer: &mut DotWriter) {
        let mut digraph = writer.digraph();
        digraph.set("compound", "true", false);

        let tokenizer = Tokenizer::new(test_version!(), "en");

        let mut config = CFGVisSettings {
            tokenizer: Some(&tokenizer),
            namespace: "".to_string(),
        };

        for frag in self {
            frag.visualize(&mut digraph, config.clone());
        }
    }
}
