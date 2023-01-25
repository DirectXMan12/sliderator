// Copyright 2021 Solly Ross

mod theme {
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::io::Write;

    #[derive(Debug, Deserialize)]
    pub(crate) struct Theme {
        pub name: String,
        pub variables: HashMap<String, String>,
        pub globals: Globals,
        pub rules: Vec<Rule>,
    }

    impl Theme {
        pub fn write<W: Write>(&self, mut w: W) -> Result<(), Box<dyn std::error::Error>> {
            writeln!(w, "/* {} theme */", self.name)?;
            writeln!(w, "pre > code > .source {{");
            for (name, val) in self.variables.iter() {
                writeln!(w, "  --{}: {};", name, val);
            }

            // TODO: errors writing optionals
            self.globals.foreground.as_ref().and_then(|fg| val_to_css(&fg)).map(|fg| writeln!(w, "  color: {};", fg));

            writeln!(w, "}}");

            for rule in self.rules.iter() {
                rule.write(&mut w)?;
            }

            Ok(())
        }
    }

    #[derive(Debug, Deserialize)]
    pub(crate) struct Globals {
        pub foreground: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    pub(crate) struct Rule {
        pub name: Option<String>,
        pub scope: String,
        pub foreground: Option<String>,
        pub background: Option<String>, 
        pub font_style: Option<String>,
    }

    impl Rule {
        pub fn write<W: Write>(&self, mut w: W) -> Result<(), Box<dyn std::error::Error>> {
            self.name.as_ref().map(|name| writeln!(w, "/* {} */", name));
            writeln!(w, "{} {{", self.scope.split(", ").map(|part| format!("pre > code > .source .{}", part)).collect::<Vec<_>>().join(", "))?;
            // TODO: errors writing optionals
            self.foreground.as_ref().and_then(|fg| val_to_css(&fg)).map(|fg| writeln!(w, "  color: {};", fg));
            self.background.as_ref().and_then(|fg| val_to_css(&fg)).map(|fg| writeln!(w, "  background: {};", fg));
            self.font_style.as_ref().and_then(|fg| val_to_css(&fg)).map(|style| style.split(" ").for_each(|opt| {
                match opt {
                    "bold" => writeln!(w, "  font-weight: bold;"),
                    "italic" => writeln!(w, "  font-style: italic;"),
                    _ => writeln!(w, "/* font option: {} */", opt),
                };
            }));
            writeln!(w, "}}");

            Ok(())
        }
    }

    pub(crate) fn val_to_css(val: &str) -> Option<String> {
        if val.starts_with("var(") && val.ends_with(")") {
            Some(format!("var(--{})", &val[4..val.len()-1]))
        } else if val.starts_with("color(") {
            None
        } else {
            Some(val.into())
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self};

    let style: theme::Theme = serde_json::from_reader(io::stdin().lock())?;

    let stdout = io::stdout();
    style.write(stdout.lock())?;

    Ok(())
}
