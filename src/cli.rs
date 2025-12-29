use std::env;

#[derive(Debug)]
pub struct Args {
    pub input: String,
    pub output: String,
    pub stream: bool,
}
#[allow(clippy::derivable_impls)]
impl Default for Args {
    fn default() -> Self {
        Self {
            input: Default::default(),
            output: Default::default(),
            stream: false,
        }
    }
}

fn print_help(program_name: &str) {
    let help_message = format!(
        r#"
https://github.com/waresnew/gif-compressor

Usage:
{} [arguments]

Mandatory arguments:
  -i FILE       Specify the input file.
  -o FILE       Specify the output file.

Optional arguments:
  --stream      Instructs the program to not store all GIF frames in memory at once. Leads to reduced peak memory usage at the cost of longer runtime.
"#,
        program_name
    );
    println!("{help_message}")
}
pub fn parse_args(mut args_raw: env::Args) -> Args {
    let program_name = args_raw.next().unwrap();
    let mut args = Args::default();
    if args_raw.len() == 0 {
        print_help(&program_name);
        std::process::exit(0);
    }
    while let Some(arg) = args_raw.next() {
        match arg.as_str() {
            "-i" => {
                args.input = args_raw.next().expect("missing input file");
            }
            "-h" | "--help" => {
                print_help(&program_name);
                std::process::exit(0);
            }
            "-o" => {
                args.output = args_raw.next().expect("missing output file");
            }
            "--stream" => {
                args.stream = true;
            }
            "--" => {
                break;
            }
            x => {
                println!("unexpected token: {}.\ntry {program_name} -h for help.", x);
                std::process::exit(0);
            }
        }
    }
    args
}
