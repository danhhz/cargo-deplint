// Copyright 2023 Daniel Harrison. All Rights Reserved.

use crate::deps::{run_lints, CargoLock, Lints};

mod deps;

fn main() {
    let args = parse_args(std::env::args()).unwrap_or_else(|err| {
        eprintln!("usage: cargo deplint path/to/Cargo.lock path/to/lints");
        eprintln!("  {}", err);
        std::process::exit(1);
    });

    let () = run(&args).unwrap_or_else(|err| {
        eprintln!("failed: {}", err);
        std::process::exit(1);
    });
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
struct Args {
    cargo_lock: String,
    lints: String,
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args, &'static str> {
    let mut args = args.into_iter();
    let _bin = args.next().ok_or("not enough args")?;
    let mut cargo_lock = args.next().ok_or("not enough args")?;
    if cargo_lock == "deplint" {
        // We were invoked as "cargo deplint", so shift all args by one.
        cargo_lock = args.next().ok_or("not enough args")?;
    }
    let lints = args.next().ok_or("not enough args")?;
    if args.next().is_some() {
        return Err("too many args");
    }
    Ok(Args { cargo_lock, lints })
}

fn run(args: &Args) -> Result<(), String> {
    let cargo_lock = std::fs::read_to_string(&args.cargo_lock)
        .map_err(|err| format!("reading Cargo.lock at {}: {}", args.cargo_lock, err))?;
    let cargo_lock: CargoLock = toml::from_str(&cargo_lock)
        .map_err(|err| format!("parsing Cargo.lock at {}: {}", args.cargo_lock, err))?;

    let lints = std::fs::read_to_string(&args.lints)
        .map_err(|err| format!("reading lints at {}: {}", args.lints, err))?;
    let lints: Lints = toml::from_str(&lints)
        .map_err(|err| format!("parsing lints at {}: {}", args.lints, err))?;

    let violations = run_lints(&cargo_lock, &lints)?;
    for violation in violations.iter() {
        eprintln!("{}", violation);
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!("{} dep lint violation(s)", violations.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args() {
        fn parse(x: &str) -> Result<Args, &'static str> {
            parse_args(x.split_ascii_whitespace().map(|x| x.to_owned()))
        }

        // cargo-deplint
        assert_eq!(parse("path/to/cargo-deplint"), Err("not enough args"));
        assert_eq!(parse("path/to/cargo-deplint foo"), Err("not enough args"));
        assert_eq!(
            parse("path/to/cargo-deplint foo bar"),
            Ok(Args {
                cargo_lock: "foo".into(),
                lints: "bar".into()
            })
        );
        assert_eq!(
            parse("path/to/cargo-deplint foo bar baz"),
            Err("too many args")
        );

        // cargo deplint
        assert_eq!(
            parse("path/to/cargo-deplint deplint"),
            Err("not enough args")
        );
        assert_eq!(
            parse("path/to/cargo-deplint deplint foo"),
            Err("not enough args")
        );
        assert_eq!(
            parse("path/to/cargo-deplint deplint foo bar"),
            Ok(Args {
                cargo_lock: "foo".into(),
                lints: "bar".into()
            })
        );
        assert_eq!(
            parse("path/to/cargo-deplint deplint foo bar baz"),
            Err("too many args")
        );
    }
}
