#![deny(clippy::all)]

use clap::{arg, crate_version, Command};
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

fn main() {
    std::process::exit(match try_main() {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("{}", e);

            1
        }
    });
}

fn fmt_io_err<T>(r: std::io::Result<T>) -> Result<T, String> {
    r.map_err(|e| format!("{}", e))
}

fn try_main() -> Result<(), String> {
    let matches = Command::new("SBNF compiler")
        .version(crate_version!())
        .about(format!("SBNF compiler {}", crate_version!()))
        .arg(arg!(quiet: -q "Do not display warnings"))
        .arg(arg!(debug: -g "Compile with debug scopes"))
        .arg(arg!(output: -o <FILE> "The file to write the compiled sublime-syntax to. \
         Defaults to $INPUT.sublime-syntax if left out. Use a single dash `-` \
         to write to stdout instead."))
        .arg(arg!(<INPUT> "The SBNF file to compile"))
        .arg(arg!(<ARGS> ... "Arguments to pass to the main and prototype rules").required(false))
        .get_matches();

    let input = matches.get_one::<String>("INPUT").unwrap();
    let output = matches.get_one::<String>("output");
    let args = matches.get_many::<String>("ARGS");

    let mut contents = String::new();
    {
        let mut file = fmt_io_err(File::open(input))?;
        fmt_io_err(file.read_to_string(&mut contents))?;
    }

    let compiler = sbnf::compiler::Compiler::default();

    let grammar = sbnf::sbnf::parse(&contents, &compiler.allocator)
        .map_err(|e| format!("{}", e.with_source(input, &contents)))?;

    // Use the base name of the input as a name hint
    let name_hint = Path::new(&input).file_stem().unwrap().to_str().unwrap();

    let options = sbnf::compiler::CompileOptions {
        name_hint: Some(name_hint),
        debug_contexts: matches.get_flag("debug"),
        arguments: args
            .map(|args| args.map(|a| a.as_str()).collect::<Vec<_>>())
            .unwrap_or_default(),
        entry_points: vec!["main", "prototype"],
    };

    let result = compiler.compile(&options, &grammar);

    match &result.result {
        Err(errors) => {
            for error in errors {
                eprintln!(
                    "{}",
                    error.with_compiler_and_source(
                        &compiler, "Error", input, &contents
                    )
                );
            }

            if !matches.get_flag("quiet") {
                for warning in result.warnings {
                    eprintln!(
                        "{}",
                        warning.with_compiler_and_source(
                            &compiler, "Warning", input, &contents
                        )
                    );
                }
            }

            Err("Compilation Failed".to_string())
        }
        Ok(syntax) => {
            if !matches.get_flag("quiet") {
                for warning in result.warnings {
                    eprintln!(
                        "{}",
                        warning.with_compiler_and_source(
                            &compiler, "Warning", input, &contents
                        )
                    );
                }
            }

            let mut output_buffer = String::new();
            syntax
                .serialize(&mut output_buffer)
                .map_err(|e| format!("{}", e))?;

            let output_path = match output.map(|s| s.as_str()) {
                Some("-") => None,
                Some(path) => Some(PathBuf::from(path)),
                None => {
                    Some(Path::new(&input).with_extension("sublime-syntax"))
                }
            };

            match output_path {
                None => {
                    print!("{}", output_buffer);
                }
                Some(path) => {
                    let mut file = fmt_io_err(File::create(path))?;
                    fmt_io_err(
                        file.write_fmt(format_args!("{}", output_buffer)),
                    )?;
                }
            }

            Ok(())
        }
    }
}
