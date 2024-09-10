mod encryptor;

use anyhow::{bail, Ok, Result};
use std::{fs::File, os::unix::fs::PermissionsExt, path::PathBuf};

const HELP: &str = "\
USAGE:
  minichacha <SUBCOMMAND> <INPUT> [OPTIONS] [OUTPUT]

FLAGS:
  -h, --help            Prints help information

OPTIONS:
  --passphrase STRING   Encryption passphrase. Omit to read from STDIN
  --output PATH         Specify output path. Omit to compute automatically

SUBCOMMANDS:
  encrypt               Encrypt the input
  decrypt               Decrypt the input

ARGS:
  <INPUT>               Path to the input file
";

enum Subcommand {
    Encrypt,
    Decrypt,
}

struct AppArgs {
    subcmd: Subcommand,
    passphrase: Option<String>,
    input: PathBuf,
    output: Option<PathBuf>,
}

fn parse_args() -> Result<AppArgs> {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        print!("{HELP}");
        std::process::exit(0);
    }

    let subcmd = match pargs.subcommand()? {
        None => {
            print!("{HELP}");
            std::process::exit(0);
        }
        Some(s) if s == "encrypt" => Subcommand::Encrypt,
        Some(s) if s == "decrypt" => Subcommand::Decrypt,
        Some(s) => bail!("Unknown subcommand \"{s}\""),
    };

    let args = AppArgs {
        subcmd,
        passphrase: pargs.opt_value_from_str("--passphrase")?,
        output: pargs.opt_value_from_os_str("--output", parse_path)?,
        input: pargs.free_from_os_str(parse_path)?,
    };

    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("warn: unused arguments left: {:?}.", remaining);
    }

    Ok(args)
}

fn parse_path(s: &std::ffi::OsStr) -> Result<PathBuf> {
    Ok(s.into())
}

fn main() -> Result<()> {
    let args = parse_args()?;

    let output = args.output.map_or_else(
        || {
            let mut output_path = args.input.clone();
            let ext = output_path.extension();

            match (ext, &args.subcmd) {
                (None, Subcommand::Encrypt) => {
                    output_path.set_extension("minichacha");
                }
                (None, Subcommand::Decrypt) => {
                    bail!("Expected 'minichacha' extension on input in decrypt mode");
                }
                (Some(s), Subcommand::Encrypt) => {
                    output_path.set_extension(format!("{}.minichacha", s.to_str().unwrap()));
                }
                (Some(s), Subcommand::Decrypt) => {
                    if s != "minichacha" {
                        bail!("Expected 'minichacha' extension on input in decrypt mode");
                    }

                    output_path.set_extension("");
                }
            }

            Ok(output_path)
        },
        Ok,
    )?;

    let passphrase = args
        .passphrase
        .map_or_else(|| Ok(rpassword::prompt_password("Input passphrase: ")?), Ok)?;

    match args.subcmd {
        Subcommand::Encrypt => {
            let input = std::fs::read(args.input)?;
            let output_file = File::create(&output)?;
            encryptor::encrypt(&input, output_file, &passphrase)?;
            std::fs::set_permissions(output, PermissionsExt::from_mode(0o444))?;
        }
        Subcommand::Decrypt => {
            let input = File::open(args.input)?;
            encryptor::decrypt(input, output, &passphrase)?;
        }
    }

    Ok(())
}
