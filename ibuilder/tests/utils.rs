#![allow(dead_code)]

use std::io::{BufRead, Write};

use failure::Error;

use ibuilder::nodes::*;
use ibuilder::*;

/// Pass a `Builder` to this function to use an interactive console inspecting the behaviour of the
/// builder. You may want to add the `--nocapture` option to see the output of this function.
pub fn interactive_console<T: 'static>(mut builder: Builder<T>) -> Result<T, Error> {
    let stdin = std::io::stdin();
    let mut iterator = stdin.lock().lines();

    loop {
        println!("\n\n\n");
        write_node(builder.to_node(), &mut std::io::stdout(), 0)?;
        let options = builder.get_options();
        println!("\n?: {}", options.query);
        for opt in &options.choices {
            println!(
                "- {}{} ({})",
                opt.text,
                if opt.needs_action { "*" } else { "" },
                opt.choice_id
            );
        }
        if options.text_input {
            println!("- textual input (> followed by the content)");
        }
        let line = iterator.next().unwrap()?;
        let input = if line.starts_with('>') {
            Input::Text(line[1..].to_string())
        } else {
            Input::Choice(line)
        };
        match builder.choose(input) {
            Ok(Some(res)) => return Ok(res),
            Ok(None) => {}
            Err(e) => println!("\n{}\n", e),
        }
    }
}

pub fn write_node<W: Write>(node: Node, w: &mut W, indent: usize) -> Result<(), Error> {
    let pad = "  ".repeat(indent);
    match node {
        Node::Composite(name, fields) => {
            w.write_all(format!("{}\n", name).as_bytes())?;
            for field in fields {
                match field {
                    FieldKind::Named(name, field) => {
                        w.write_all(format!("{}- {}: ", pad, name).as_bytes())?;
                        write_node(field, w, indent + 1)?;
                    }
                    FieldKind::Unnamed(field) => {
                        w.write_all(format!("{}- ", pad).as_bytes())?;
                        write_node(field, w, indent + 1)?;
                    }
                }
            }
        }
        Node::Leaf(field) => match field {
            Field::String(content) => w.write_all(format!("{}\n", content).as_bytes())?,
            Field::Missing => w.write_all(b"missing\n")?,
        },
    }
    Ok(())
}
