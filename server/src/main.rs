#![deny(
    anonymous_parameters,
    bare_trait_objects,
    clippy::clone_on_ref_ptr,
    clippy::float_cmp_const,
    clippy::if_not_else,
    clippy::unwrap_used
)]
#![cfg_attr(
    debug_assertions,
    allow(
        dead_code,
        unused_imports,
        unused_variables,
        clippy::match_single_binding,
    )
)]
#![cfg_attr(
    not(debug_assertions),
    deny(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::dbg_macro,
        clippy::indexing_slicing,
    )
)]

use anyhow::Result;
use futures::channel::mpsc;
use futures::future::FutureExt;
use futures::stream::StreamExt;
use structopt::StructOpt;
use tokio::net;

mod cli;
mod session;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let options = cli::Options::from_args();

    let listener = net::TcpListener::bind((options.ip.as_str(), options.port)).await?;
    log::info!("Listening on {}", listener.local_addr()?);

    let mut ctrlc = Box::pin(tokio::signal::ctrl_c().fuse());

    let (updates_send, mut updates_recv) = mpsc::channel(256);

    loop {
        futures::select! {
            _ = ctrlc => {
                log::info!("Stopping server");
                return Ok(());
            },
            conn = listener.accept().fuse() => match conn {
                Ok((conn, addr)) => {
                    log::debug!("Connection from {}", addr);
                    let updates_send = updates_send.clone();
                    tokio::spawn(async move {
                        if let Err(err) = session::handle(conn, addr, updates_send).await {
                            log::error!("Error handling connection from {}: {}", addr, err);
                        }
                    });
                }
                Err(err) => {
                    log::error!("Error accepting connection: {}", err);
                }
            },
            update = updates_recv.next() => {


            match update.expect("All update senders died") {
                session::Update::NewSession { id, name, identity: _ } => {
                    log::info!("Session {} ({}) joined the server", id, name);
                }
                session::Update::CloseSession { id} => {
                    log::info!("Session {} closed", id);
                }
            }}
        }
    }
}
