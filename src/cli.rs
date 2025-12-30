use std::env;

#[derive(Debug)]
pub struct Args {
    pub input: String,
    pub output: String,
    pub stream: bool,
    pub transparency_threshold: u32,
}
#[allow(clippy::derivable_impls)]
impl Default for Args {
    fn default() -> Self {
        Self {
            input: Default::default(),
            output: Default::default(),
            stream: false,
            transparency_threshold: 5,
        }
    }
}

fn get_help_msg(program_name: &str) -> String {
    format!(
        r#"
https://github.com/waresnew/gif-compressor

Usage:
{} [arguments]

  -i, --input FILE               (Mandatory) Specify the input file path.
  -o, --output FILE              (Mandatory) Specify the output file path.
  -h, --help                     Prints this help message.
  -s, --stream                   Instructs the program to not store all GIF frames in memory at once. Leads to reduced peak memory usage at the cost of longer runtime.
  -t, --transparency INTEGER     Specify a non-negative colour distance threshold for transparency optimization. Default: 5
"#,
        program_name
    )
}
pub fn parse_args(mut args_raw: env::Args) -> Args {
    let program_name = args_raw.next().unwrap();
    let help_msg = get_help_msg(&program_name);
    let mut args = Args::default();
    if args_raw.len() == 0 {
        eprintln!("{help_msg}");
        std::process::exit(1);
    }
    while let Some(arg) = args_raw.next() {
        match arg.as_str() {
            "-i" | "--input" => {
                args.input = args_raw.next().unwrap_or_else(|| {
                    eprintln!("missing input path");
                    std::process::exit(1);
                })
            }
            "-h" | "--help" => {
                println!("{help_msg}");
                std::process::exit(0);
            }
            "-o" | "--output" => {
                args.output = args_raw.next().unwrap_or_else(|| {
                    eprintln!("missing output path");
                    std::process::exit(1);
                });
            }
            "--stream" | "-s" => {
                args.stream = true;
            }
            "--transparency" | "-t" => {
                args.transparency_threshold = args_raw
                    .next()
                    .unwrap_or_else(|| {
                        eprintln!("missing transparency threshold");
                        std::process::exit(1);
                    })
                    .parse()
                    .unwrap_or_else(|e| {
                        eprintln!("failed to parse transparency threshold value: {e}");
                        std::process::exit(1);
                    })
            }
            "--" => {
                break;
            }
            x => {
                eprintln!("unexpected token: {}.\ntry {program_name} -h for help.", x);
                std::process::exit(1);
            }
        }
    }
    if args.input.is_empty() || args.output.is_empty() {
        eprintln!("input or output path argument is missing. run {program_name} -h for help.");
        std::process::exit(1);
    }
    args
}
