use clap::{Parser, Subcommand};
use shared::node_config::NodeConfig;
use std::{
    io::{BufRead, BufReader, Error, ErrorKind},
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::Duration,
};

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
async fn main() -> Result<(), Error> {
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

            let node_child = Command::new("env")
                .args([&format!("PORT={}", node_port), "node", "node_build"])
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;

            let node_stdout = node_child.stdout.ok_or_else(|| {
                Error::new(ErrorKind::Other, "Could not capture standard output.")
            })?;
            let mut stdout_reader = BufReader::new(node_stdout);

            let node_stderr = node_child
                .stderr
                .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard error."))?;
            let mut stderr_reader = BufReader::new(node_stderr);

            let (node_ready_sender, node_ready_receiver) = mpsc::channel();

            // TODO: Should panic on error?
            let stderr_thread = thread::spawn(move || {
                let mut line = String::new();

                loop {
                    let stderr_result = stderr_reader.read_line(&mut line);
                    match stderr_result {
                        Ok(0) => break,
                        Ok(_) => {
                            print!("[NODE SERVER STDERR] {}", line);
                            line.clear();
                        }
                        Err(e) => {
                            eprintln!("[NODE SERVER STDERR] Error reading stderr: {}", e);
                            break;
                        }
                    }
                }
            });

            let stdout_thread = thread::spawn(move || {
                let mut line = String::new();
                let mut node_ready_sender = Some(node_ready_sender);

                loop {
                    let stdout_result = stdout_reader.read_line(&mut line);
                    match stdout_result {
                        Ok(0) => break,
                        Ok(_) => {
                            if line.contains("Listening on ") {
                                if let Some(sender) = node_ready_sender.take() {
                                    let _ = sender.send(());
                                }
                            }
                            print!("[NODE SERVER STDOUT] {}", line);
                            line.clear();
                        }
                        Err(e) => {
                            eprintln!("[NODE SERVER STDOUT] Error reading stdout: {}", e);
                            break;
                        }
                    }
                }
            });

            match node_ready_receiver.recv_timeout(Duration::from_secs(30)) {
                Ok(()) => {}
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    return Err(Error::new(
                        ErrorKind::TimedOut,
                        "Timed out waiting for Node server readiness.",
                    ));
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(Error::new(
                        ErrorKind::BrokenPipe,
                        "Node server exited before signaling readiness.",
                    ));
                }
            }

            let _ = api::build_rocket(NodeConfig { port: node_port })
                .configure(rocket::Config::figment().merge(("port", server_port)))
                .launch()
                .await;

            stdout_thread.join().unwrap();
            stderr_thread.join().unwrap();
        }
    }

    Ok(())
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
