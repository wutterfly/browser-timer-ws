#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

mod data;
mod err;
mod message;
mod ping;
mod server;
mod time;

use data::{Distributer, WriteOn};
use env_logger::Env;
use message::EventTyp;
use server::Server;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize global logger
    env_logger::Builder::from_env(Env::default().default_filter_or("trace"))
        .format(|buf, record| {
            writeln!(
                buf,
                "[{:<5}] -- [{}]  {}",
                record.level(),
                record.target(),
                record.args()
            )
        })
        .filter_module("tokio_tungstenite", log::LevelFilter::Off)
        .filter_module("tungstenite", log::LevelFilter::Off)
        .init();

    // build async runtime
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    // prepare data output
    let dist = Distributer::new(
        Some(WriteOn::Filter(Arc::new(|data| {
            data.key_code == 27 && data.typ == EventTyp::KeyUp
        }))),
        PathBuf::from("./output"),
        "./key_data",
    )?;

    // create websocket server
    let server = Server::new(4);

    let port = 8021;

    // start server
    rt.block_on(server.start_server(dist, port))?;

    Ok(())
}
