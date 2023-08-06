use clap::*;

fn main() {
    let matches = Command::new("tcl")
        .version("1.0")
        .author("Aiden Teng")
        .about("template cli") // requires `cargo` feature
        .arg(
            Arg::new("hello")
                .long("hello")
                .required(false)
                .default_value("world")
                .value_parser(value_parser!(String)),
        )
        .subcommand_required(true)
        .subcommand(
            Command::new("hi")
                .about("hi")
                .arg(
                    Arg::new("number")
                        .help(r#"--name "just a argument""#)
                        .short('n')
                        .long("number")
                        .required(false)
                        .default_value("0")
                        .value_parser(value_parser!(usize)),
                )
        ).get_matches();

    match matches.subcommand() {
        Some(("hi", matches)) => {
            let number = matches.get_one::<usize>("number").unwrap();
            println!("hi {}", number);
        }
        _ => unreachable!("No subcommand was used"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BIN_NAME: &str = "main";

    #[test]
    fn main() {
        let mut cmd = assert_cmd::Command::cargo_bin(BIN_NAME).unwrap();
        cmd.arg("help").assert().success();
    }
}
