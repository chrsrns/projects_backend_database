use clap::{Parser, Subcommand};
use shared::node_config::NodeConfig;
use std::{
    io::{BufRead, BufReader, Error, ErrorKind},
    path::PathBuf,
    process::{Command, Stdio},
    sync::{Arc, mpsc},
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
    Serve {
        #[arg(long = "port", short = 'p')]
        port: Option<u16>,
        #[arg(long = "node-port", short = 'n')]
        node_port: Option<u16>,
        #[arg(long = "skip-node-run", short = 's', default_value_t = false)]
        skip_node_run: bool,
    },
}

fn resolve_node_build_dir() -> Result<PathBuf, Error> {
    let current_dir = std::env::current_dir()?;
    let node_build_dir = current_dir.join("node_build");

    if !node_build_dir.exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!(
                "Node server directory was not found at {}",
                node_build_dir.display()
            ),
        ));
    }

    if !node_build_dir.is_dir() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!(
                "Node server path is not a directory: {}",
                node_build_dir.display()
            ),
        ));
    }

    for required_path in ["index.js", "handler.js", "server"] {
        let required_path = node_build_dir.join(required_path);
        if !required_path.exists() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Node server build is missing required path: {}",
                    required_path.display()
                ),
            ));
        }
    }

    if !node_build_dir.join("server").is_dir() {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "Node server build path is not a directory: {}",
                node_build_dir.join("server").display()
            ),
        ));
    }

    Ok(node_build_dir)
}

fn log_node_server_warning_banner(message: &str) {
    let border = "=".repeat(88);
    log::warn!("{}", border);
    log::warn!("NODE SERVER WARNING");
    log::warn!("{}", message);
    log::warn!("{}", border);
}

#[rocket::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    api::init_logging();

    infrastructure::run_migrations_once()
        .inspect_err(|e| log::error!("Failed to run database migrations: {}", e))?;

    // Print version on startup
    println!(
        "Rust Profile Management Backend v{}",
        env!("CARGO_PKG_VERSION")
    );

    match cli.command {
        Commands::Serve {
            port,
            node_port,
            skip_node_run,
        } => {
            let rocket_port_env = std::env::var("ROCKET_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok());
            let server_port = port.or(rocket_port_env).unwrap_or(8000);

            let node_port = node_port.unwrap_or_else(|| {
                loop {
                    let port = rand::random::<u16>() % (65535 - 1024) + 1024;
                    if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
                        break port;
                    }
                }
            });

            let mut stdout_thread = None;
            let mut stderr_thread = None;

            if !skip_node_run {
                let node_build_dir = resolve_node_build_dir().map_err(|err| {
                    log_node_server_warning_banner(&format!(
                        "The Rust server could not load the Node server under 'node_build'. {}",
                        err
                    ));
                    err
                })?;
                let port_env = format!("PORT={}", node_port);
                let node_build_dir_arg = node_build_dir.to_string_lossy().into_owned();
                let node_child = Command::new("env")
                    .args([port_env.as_str(), "node", node_build_dir_arg.as_str()])
                    .stderr(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .map_err(|err| {
                        log_node_server_warning_banner(&format!(
                            "The Rust server could not load the Node server under '{}'. {}",
                            node_build_dir.display(),
                            err
                        ));
                        err
                    })?;

                let node_stdout = node_child
                    .stdout
                    .ok_or_else(|| Error::other("Could not capture standard output."))?;
                let mut stdout_reader = BufReader::new(node_stdout);

                let node_stderr = node_child
                    .stderr
                    .ok_or_else(|| Error::other("Could not capture standard error."))?;
                let mut stderr_reader = BufReader::new(node_stderr);

                let (node_ready_sender, node_ready_receiver) = mpsc::channel();

                // TODO: Should panic on error?
                stderr_thread = Some(thread::spawn(move || {
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
                }));

                stdout_thread = Some(thread::spawn(move || {
                    let mut line = String::new();
                    let mut node_ready_sender = Some(node_ready_sender);

                    loop {
                        let stdout_result = stdout_reader.read_line(&mut line);
                        match stdout_result {
                            Ok(0) => break,
                            Ok(_) => {
                                if line.contains("Listening on ") {
                                    let _ = node_ready_sender.take().map(|sender| sender.send(()));
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
                }));

                match node_ready_receiver.recv_timeout(Duration::from_secs(30)) {
                    Ok(()) => {}
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        log_node_server_warning_banner(
                            "The Rust server could not load the Node server under 'node_build'. Timed out waiting for Node server readiness.",
                        );
                        return Err(Error::new(
                            ErrorKind::TimedOut,
                            "Timed out waiting for Node server readiness.",
                        ));
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        log_node_server_warning_banner(
                            "The Rust server could not load the Node server under 'node_build'. Node server exited before signaling readiness.",
                        );
                        return Err(Error::new(
                            ErrorKind::BrokenPipe,
                            "Node server exited before signaling readiness.",
                        ));
                    }
                }
            }

            let api_key = std::env::var("GEMINI_API_KEY").ok();
            let model =
                std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-3.5-flash".to_string());
            let llm_client: Arc<dyn api::llm::LlmClient + Send + Sync> =
                Arc::new(api::llm::GeminiClient::new(api_key, model));

            let _ = api::build_rocket_with_llm_client(
                api::realtime::Hub::new(),
                NodeConfig { port: node_port },
                llm_client,
            )
            .configure(rocket::Config::figment().merge(("port", server_port)))
            .launch()
            .await;

            if let Some(stdout_thread) = stdout_thread {
                stdout_thread.join().unwrap();
            }

            if let Some(stderr_thread) = stderr_thread {
                stderr_thread.join().unwrap();
            }
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

    #[test]
    fn test_version_from_cargo() {
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty());
        assert_eq!(version, "0.5.0");
    }

    #[test]
    fn test_cli_version_flag_exists() {
        // This test verifies that the Cli struct has version enabled
        // The actual --version functionality is handled by clap
        use clap::error::ErrorKind;
        let cli_result = Cli::try_parse_from(["test", "--version"]);
        match cli_result {
            Err(err) => {
                // --version should cause clap to display version and exit
                assert!(matches!(err.kind(), ErrorKind::DisplayVersion));
            }
            Ok(_) => panic!("--version should cause clap to exit with DisplayVersion error"),
        }
    }
}
