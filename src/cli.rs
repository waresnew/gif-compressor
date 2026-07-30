use clap::Parser;
use clap_verbosity_flag::{Verbosity, WarnLevel};

#[derive(Debug, Clone, Parser)]
#[command(version, about, long_about=None)]
pub struct Cli {
    /// The input file path.
    #[arg(short, long)]
    pub input: String,

    /// The output file path.
    #[arg(short, long)]
    pub output: String,

    /// How many frames to send to the GPU at a time. Setting it to 0 will use as much memory as your
    /// GPU allows in a storage buffer.
    #[arg(short, long, default_value_t = 0)]
    pub chunk_size: usize,

    /// Specify a non-negative colour distance threshold for transparency optimization.
    #[arg(short, long, default_value_t = 5)]
    pub transparency_threshold: u32,

    #[command(flatten)]
    pub verbosity: Verbosity<WarnLevel>,
}
