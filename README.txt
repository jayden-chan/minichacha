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
