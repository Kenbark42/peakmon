use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "peakmon", version, about = "A real-time terminal system monitor")]
pub struct Config {
    /// Metrics refresh interval in milliseconds
    #[arg(short, long, default_value_t = 1000, value_parser = clap::value_parser!(u64).range(250..=10000))]
    pub refresh_rate: u64,
}
