mod item;
mod server;
mod session;

use clap::Parser;
use color_eyre::Report;

#[derive(Parser, Debug)]
#[clap(about, long_about, version)]
pub struct Args {
    #[clap(short, long, default_value = "23333")]
    port: u16,
}

impl Args {
    pub fn port(&self) -> u16 {
        self.port
    }
}

#[tokio::main]
async fn main() -> Result<(), Report> {
    let args = Args::parse();

    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "info");
        }
    }

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .pretty()
        .init();

    let server = server::Server::setup(args.port()).await?;
    server.run().await;

    Ok(())
}
