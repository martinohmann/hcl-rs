fn main() -> Result<(), lexopt::Error> {
    let args = Args::parse()?;

    match args.parser {
        Parser::Edit => {
            let _content = hcl_edit::parser::parse_body(&args.input).unwrap();
            #[cfg(debug_assertions)] // Don't interefere with profiling
            dbg!(_content);
        }
        Parser::Normal => {
            let _content = hcl::parse(&args.input).unwrap();
            #[cfg(debug_assertions)] // Don't interefere with profiling
            dbg!(_content);
        }
    }
    Ok(())
}

struct Args {
    parser: Parser,
    input: String,
}

impl Args {
    fn parse() -> Result<Self, lexopt::Error> {
        use lexopt::prelude::*;

        let mut parser = Parser::Normal;
        let tests = testdata::load().unwrap();
        let mut paths = tests.iter().map(|t| t.name()).collect::<Vec<_>>();
        paths.sort();
        let mut path = paths[0].clone();

        let mut args = lexopt::Parser::from_env();
        while let Some(arg) = args.next()? {
            match arg {
                Long("input") => {
                    let value = args.value()?.parse_with(|p| {
                        if !paths.iter().any(|path| path == p) {
                            return Err(format!("expected one of {}, got {}", paths.join(", "), p));
                        }
                        Ok(p.to_owned())
                    })?;
                    path = value;
                }
                Long("parser") => {
                    let value = args.value()?;
                    parser = match &value.to_str() {
                        Some("edit") => Parser::Edit,
                        Some("normal") => Parser::Normal,
                        _ => {
                            return Err(lexopt::Error::UnexpectedValue {
                                option: "parser".to_owned(),
                                value: value.clone(),
                            });
                        }
                    };
                }
                _ => return Err(arg.unexpected()),
            }
        }

        let input = tests
            .iter()
            .find(|t| t.name() == path)
            .unwrap()
            .input
            .clone();

        Ok(Self { parser, input })
    }
}

enum Parser {
    Edit,
    Normal,
}
