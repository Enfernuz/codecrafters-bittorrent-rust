use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(next_line_help = true)]
pub struct CliArgs {
    
    #[arg(long)]
    pub decode: Option<String>,

    #[arg(long)]
    pub info: Option<String>,
}