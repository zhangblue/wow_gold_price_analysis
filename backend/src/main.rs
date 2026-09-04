use clap::Parser;

#[derive(clap::Parser)]
struct Cli {
    #[arg(long)]
    host: std::net::IpAddr,
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..))]
    port: u16,
}

fn main() {
    let _cli = Cli::parse();
}
