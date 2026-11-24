//! Complete-clip MIDI CRUD and focused native development probes.
mod cli_arranger;
mod cli_automation;
mod cli_connection;
mod cli_controller;
mod cli_cues;
mod cli_instrument;
mod cli_mix;
mod cli_osc;
mod osc_session;
use clap::{Parser, Subcommand};
use notes_transformer::{
    adapter,
    session::{Sessions, serve},
};
use std::{error::Error, net::Ipv4Addr, path::PathBuf};
#[derive(Parser)]
#[command(name = "midi-bridge")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    Serve {
        #[arg(long, default_value_t = 8788)]
        port: u16,
        #[arg(long,default_value_t=notes_transformer::api::DEFAULT_PORT)]
        api_port: u16,
        /// Enable OSC and send to this local `DrivenByMoss` command port.
        #[arg(long, value_parser = clap::value_parser!(u16).range(1..))]
        osc_port: Option<u16>,
        /// Local UDP port used by `DrivenByMoss` for feedback.
        #[arg(long, requires = "osc_port", value_parser = clap::value_parser!(u16).range(1..))]
        osc_feedback_port: Option<u16>,
    },
    /// Send a raw `DrivenByMoss` OSC message through the running bridge.
    Osc(cli_osc::Osc),
    Controller(cli_controller::Controller),
    Adapter {
        #[command(subcommand)]
        host: Host,
    },
}
#[derive(Subcommand)]
enum Host {
    Bitwig {
        #[command(subcommand)]
        command: Bitwig,
    },
}
#[derive(Subcommand)]
enum Bitwig {
    Connection(cli_connection::Connection),
    Probe {
        #[command(subcommand)]
        command: Probe,
    },
}
#[derive(Subcommand)]
enum Probe {
    ReloadAdapter {
        #[arg(long)]
        driver: Option<String>,
        #[arg(long, requires = "driver")]
        refresh: bool,
    },
    Arranger {
        #[arg(long)]
        input: PathBuf,
    },
    Authority,
    Construction {
        #[command(subcommand)]
        command: Construction,
    },
    Reload,
    Selection,
    Freeze,
    Arm {
        #[arg(long)]
        document: String,
    },
    Apply {
        #[arg(long)]
        plan: PathBuf,
    },
    Undo,
    Redo,
}
#[derive(Subcommand)]
enum Construction {
    Native {
        #[arg(long)]
        project_uuid: String,
        #[arg(long)]
        revision: String,
    },
}
fn probe(command: Probe) -> Result<(), Box<dyn Error>> {
    let result = match command {
        Probe::ReloadAdapter { driver, refresh } => {
            adapter::reload_adapter(driver.as_deref(), refresh)?
        }
        Probe::Arranger { input } => cli_arranger::probe(&input)?,
        Probe::Authority => adapter::authority_probe()?,
        Probe::Construction {
            command:
                Construction::Native {
                    project_uuid,
                    revision,
                },
        } => adapter::construction(&project_uuid, &revision)?,
        Probe::Arm { document } => adapter::probe("arm", None, Some(&document))?,
        Probe::Apply { plan } => adapter::probe("apply", Some(&plan), None)?,
        other => adapter::probe(
            match other {
                Probe::Reload => "reload",
                Probe::Selection => "selection",
                Probe::Freeze => "freeze",
                Probe::Undo => "undo",
                Probe::Redo => "redo",
                _ => unreachable!(),
            },
            None,
            None,
        )?,
    };
    println!("{}", serde_json::to_string(&result)?);
    if !result["error"].is_null() || result["observed"]["status"] == "failed" {
        return Err("Native probe failed; inspect structured result".into());
    }
    Ok(())
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with_writer(std::io::stderr)
        .init();
    match Cli::parse().command {
        Commands::Serve {
            port,
            api_port,
            osc_port,
            osc_feedback_port,
        } => {
            let osc = osc_port.map(|port| {
                (
                    port,
                    osc_feedback_port.unwrap_or(osc_session::DEFAULT_FEEDBACK_PORT),
                )
            });
            serve_controller(port, api_port, osc).await?;
        }
        Commands::Osc(args) => cli_osc::run(args).await?,
        Commands::Controller(args) => cli_controller::run(args).await?,
        Commands::Adapter {
            host: Host::Bitwig { command },
        } => match command {
            Bitwig::Connection(args) => {
                if !cli_connection::run(args)? {
                    std::process::exit(1)
                }
            }
            Bitwig::Probe { command } => probe(command)?,
        },
    }
    Ok(())
}
async fn serve_controller(
    port: u16,
    api_port: u16,
    osc: Option<(u16, u16)>,
) -> std::io::Result<()> {
    let native = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await?;
    let http = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, api_port)).await?;
    let (osc_routes, osc_listener) = if let Some((target, feedback)) = osc {
        if target == feedback {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "OSC command and feedback ports must be distinct",
            ));
        }
        let (dispatch, listener) = osc_session::Listener::bind(target, feedback).await?;
        (osc_session::router(dispatch), Some(listener))
    } else {
        (axum::Router::new(), None)
    };
    tracing::info!(adapter=%native.local_addr()?, api=%http.local_addr()?, "Controller service ready");
    let sessions = Sessions::default();
    let (stop, stopped) = tokio::sync::watch::channel(false);
    let api_stop = stopped.clone();
    let native_stop = stopped.clone();
    let http_service =
        notes_transformer::api::serve_with_routes(http, sessions.clone(), osc_routes, async move {
            let mut stopped = api_stop;
            let _ = stopped.changed().await;
        });
    let native_service = serve(native, sessions, async move {
        let mut stopped = native_stop;
        let _ = stopped.changed().await;
    });
    let signal = async {
        #[cfg(unix)]
        {
            let mut terminate =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
            tokio::select! { result = tokio::signal::ctrl_c() => result?, _ = terminate.recv() => {} }
        }
        #[cfg(not(unix))]
        tokio::signal::ctrl_c().await?;
        let _ = stop.send(true);
        Ok::<(), std::io::Error>(())
    };
    let osc_service = async {
        if let Some(listener) = osc_listener {
            listener.run(stopped).await?;
        }
        Ok::<(), std::io::Error>(())
    };
    tokio::try_join!(http_service, native_service, osc_service, signal)?;
    Ok(())
}
