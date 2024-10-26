use std::fmt::{Debug, Display};

use dot_writer::{DotWriter, Scope};

use equidistributed_colors::EquiColor;

static mut COLORS: Option<EquiColor> = None;

pub fn next_color() -> String {
    let color = unsafe {
        if COLORS.is_none() {
            COLORS = Some(EquiColor::new(0.5, 0.5))
        }

        COLORS.unwrap().next().unwrap()
    };

    format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}

#[derive(Clone)]
pub enum Coloring {
    None,
    Fixed(String),
    Random,
}

impl Display for Coloring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            Coloring::None => "black".to_string(),
            Coloring::Fixed(s) => s.clone(),
            Coloring::Random => next_color(),
        })
    }
}

impl Coloring {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn is_random(&self) -> bool {
        matches!(self, Self::Random)
    }

    pub fn is_fixed(&self) -> bool {
        matches!(self, Self::Fixed(_))
    }
}

pub trait Visualize<T: Sized> {
    fn visualize(&self, context: &mut Scope, config: T);
}

pub trait Graphviz {
    fn graphviz(&self, writer: &mut DotWriter);
}

impl Debug for dyn Graphviz {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output_bytes = Vec::new();
        {
            let mut writer = DotWriter::from(&mut output_bytes);

            self.graphviz(&mut writer);
        }

        f.write_str(&String::from_utf8(output_bytes).unwrap())
    }
}

pub fn escape(string: &str) -> String {
    string
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('"', "\\\"")
}
