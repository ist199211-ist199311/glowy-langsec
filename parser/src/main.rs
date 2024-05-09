use std::{env, fs, io};

use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFile,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use parser::{parse, Diagnostics, ErrorDiagnosticInfo};

fn main() {
    let path = env::args().nth(1);

    let input = if let Some(path) = &path {
        fs::read_to_string(path).expect("Failed to read file at specified path")
    } else {
        io::read_to_string(io::stdin()).expect("Failed to read input from stdin")
    };

    match parse(&input) {
        Ok(root) => println!("{root:#?}"),
        Err(err) => show_error(&path, &input, err.diagnostics()),
    }
}

fn show_error(path: &Option<String>, input: &str, info: ErrorDiagnosticInfo) {
    let file = SimpleFile::new(path.as_deref().unwrap_or("<stdin>"), input);

    let diagnostic = Diagnostic::error()
        .with_code(info.code)
        .with_message(info.overview)
        .with_labels(vec![
            Label::primary((), info.context.location()).with_message(info.details)
        ])
        .with_notes(vec![concat!(
            "help: if you're sure your Go syntax is correct, ",
            "this parser may not support that construct"
        )
        .to_owned()]);

    let writer = StandardStream::stderr(ColorChoice::Auto);
    let config = term::Config::default();

    term::emit(&mut writer.lock(), &config, &file, &diagnostic).expect("Failed to show error");
}
