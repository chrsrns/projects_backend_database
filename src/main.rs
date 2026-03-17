use clap::{Parser, Subcommand};
use shared::node_config::NodeConfig;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Serve { port: Option<u16> },
}

#[rocket::main]
async fn main() {
    let cli = Cli::parse();

    api::init_logging();

    match cli.command {
        Commands::Serve { port } => {
            let rocket_port_env = std::env::var("ROCKET_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok());
            let server_port = port.or(rocket_port_env).unwrap_or(8000);

            let node_port: u16 = loop {
                let port = rand::random::<u16>() % (65535 - 1024) + 1024;
                if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
                    break port;
                }
            };

            let _ = api::build_rocket(NodeConfig { port: node_port })
                .configure(rocket::Config::figment().merge(("port", server_port)))
                .launch()
                .await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
