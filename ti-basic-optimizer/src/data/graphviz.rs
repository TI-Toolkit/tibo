use dot_writer::{DotWriter, Scope};

pub trait Visualize<T: Sized> {
    fn visualize(&self, context: &mut Scope, config: T);
}

pub trait Graphviz {
    fn graphviz(&self, writer: &mut DotWriter);

    fn dbg(&self) {
        let mut output_bytes = Vec::new();
        {
            let mut writer = DotWriter::from(&mut output_bytes);

            self.graphviz(&mut writer);
        }

        print!("{}", String::from_utf8(output_bytes).unwrap())
    }
}

pub fn escape(string: &str) -> String {
    string
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('"', "\\\"")
}
