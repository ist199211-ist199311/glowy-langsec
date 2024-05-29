use std::{env, fs, io};

use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use glowy::{analyze_files, errors::AnalysisError};
use parser::Diagnostics;

fn main() {
    let path = env::args().nth(1);

    let mut files: Vec<(String, String)> = Vec::new();

    if let Some(path) = path {
        let content = fs::read_to_string(&path).expect("Failed to read file at specified path");
        files.push((path, content));
    } else {
        let content = io::read_to_string(io::stdin()).expect("Failed to read input from stdin");
        files.push(("<stdin>".to_string(), content));
    };

    let mut codespan_files = SimpleFiles::new();
    let files_to_analyze = files.iter().map(|(file_name, contents)| {
        (
            codespan_files.add(file_name.as_str(), contents.as_str()),
            contents.as_str(),
        )
    });

    match analyze_files(files_to_analyze) {
        Ok(_) => println!("Analysis succeeded with no errors found"),
        Err(errors) => {
            let writer = StandardStream::stderr(ColorChoice::Auto);
            let config = term::Config::default();

            for error in errors {
                let diagnostic = get_diagnostic_for_error(error, &codespan_files);
                term::emit(&mut writer.lock(), &config, &codespan_files, &diagnostic)
                    .expect("Failed to show error");
            }
        }
    }
}

fn get_diagnostic_for_error<'a>(
    error: AnalysisError<'a>,
    files: &SimpleFiles<&'a str, &'a str>,
) -> Diagnostic<usize> {
    macro_rules! s {
        ($lit:expr) => {
            $lit.to_owned()
        };
    }

    match error {
        AnalysisError::Parsing { file, error } => {
            let info = error.diagnostics();
            let location = if let Some(ctx) = info.context {
                ctx.location()
            } else {
                // default to last char to represent eof;
                // note that this might return an empty range if input is empty
                let input = files.get(file).expect("file to exist").source();
                input.len().saturating_sub(1)..input.len()
            };
            Diagnostic::error()
                .with_code(info.code)
                .with_message(info.overview)
                .with_labels(vec![
                    Label::primary(file, location).with_message(info.details)
                ])
                .with_notes(vec![concat!(
                    "help: if you're sure your Go syntax is correct, ",
                    "this parser may not support that construct"
                )
                .to_owned()])
        }
        AnalysisError::DataFlowAssignment {
            file,
            symbol,
            sink_label,
            expression_label,
        } => {
            Diagnostic::error()
                .with_code("E001")
                .with_message(s!("insecure data flow to sink during assigment"))
                .with_labels(vec![
                    Label::primary(file, symbol.location()).with_message(format!(
                        "sink `{}` has label {}, but the expression being assigned to it has \
                         label {}",
                        symbol.content(),
                        sink_label,
                        expression_label
                    )),
                    // TODO: when the error supports it, show where the labels of the expression
                    // come from
                ])
        }
        AnalysisError::DataFlowFuncCall => todo!(),
        AnalysisError::UnknownSymbol { file, symbol } => Diagnostic::warning()
            .with_code("W001")
            .with_message(s!("symbol not found"))
            .with_labels(vec![Label::primary(file, symbol.location()).with_message(
                format!("symbol `{}` has not been declared", symbol.content()),
            )]),
        AnalysisError::Redeclaration {
            file,
            prev_symbol,
            new_symbol,
        } => Diagnostic::warning()
            .with_code("W002")
            .with_message(s!("symbol redeclaration"))
            .with_labels(vec![
                Label::primary(file, new_symbol.location()).with_message(format!(
                    "symbol `{}` has already been declared",
                    new_symbol.content()
                )),
                Label::secondary(file, prev_symbol.location()).with_message(format!(
                    "previous declaraction of `{}` is here",
                    prev_symbol.content()
                )),
            ])
            .with_notes(vec![concat!(
                "note: for static analysis purposes, the redeclaration is ",
                "taken into account, replacing the previous symbol"
            )
            .to_owned()]),
    }
}
