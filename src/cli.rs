use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(version, about, long_about=None)]
pub struct Cli {
    /// The input file path.
    #[arg(short, long)]
    pub input: String,

    /// The output file path.
    #[arg(short, long)]
    pub output: String,

    /// Instructs the program to not store all GIF frames in memory at once. Leads to reduced peak memory usage at the cost of longer runtime.
    #[arg(short, long, default_value_t = false)]
    pub stream: bool,

    /// Specify a non-negative colour distance threshold for transparency optimization.
    #[arg(short, long, default_value_t = 5)]
    pub transparency_threshold: u32,
}
