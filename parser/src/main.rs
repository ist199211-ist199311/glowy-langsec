use std::io;

use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFile,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use parser::{parse, Span};

fn main() {
    let input = io::read_to_string(io::stdin()).expect("Failed to read input from stdin");

    match parse(&input) {
        Ok(root) => println!("{root:#?}"),
        Err(Some(ctx)) => show_error(&input, &ctx),
        Err(None) => eprintln!("ERROR: Something went wrong while parsing!"),
    }
}

fn show_error(input: &str, ctx: &Span) {
    // TODO: show more detailed/useful error messages

    let file = SimpleFile::new("<stdin>", input);

    let diagnostic = Diagnostic::error()
        .with_code("E042")
        .with_message("invalid syntax")
        .with_labels(vec![Label::primary(
            (),
            ctx.location_offset()..(ctx.location_offset() + ctx.len() + 1),
        )
        .with_message("could not parse this segment")])
        .with_notes(vec![concat!(
            "help: if you're sure your Go syntax is correct, ",
            "this parser may not support that construct"
        )
        .to_owned()]);

    let writer = StandardStream::stderr(ColorChoice::Auto);
    let config = term::Config::default();

    term::emit(&mut writer.lock(), &config, &file, &diagnostic).expect("Failed to show error");
}
