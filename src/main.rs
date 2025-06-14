use std::process::Command;

use axum::extract::Query;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use clap::Parser;
use serde::Deserialize;
use tokio::io::AsyncBufReadExt;
use tokio::net::TcpListener;
use tracing::info;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct WebArgs {
    #[arg(short, long)]
    #[arg(default_value_t = 4002)]
    #[arg(value_parser = clap::value_parser!(u16).range(0..=65535))]
    port: u16,
}

#[derive(Debug, Deserialize)]
struct QueryParams {
    size: Option<Size>,
}

#[derive(Debug, Deserialize)]
enum Size {
    Short,
    Long,
}

#[tokio::main]
async fn main() {
    // set log level with env variable RUST_LOG
    tracing_subscriber::fmt::init();
    // print all log levels
    // tracing_subscriber::registry().with(fmt::layer()).init();

    let web_args = WebArgs::parse();

    let router = Router::new()
        .route("/", get(home))
        .route("/english", get(english_word));

    let address_port = format!("0.0.0.0:{}", web_args.port);
    let listener = TcpListener::bind(address_port).await.unwrap();
    info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, router).await.unwrap();
}

async fn home() -> &'static str {
    "get an english word with definition\n"
}

async fn english_word(Query(params): Query<QueryParams>) -> impl IntoResponse {
    let size = params.size.unwrap_or(Size::Long);
    let output = Command::new("/usr/games/fortune")
        .arg("magoosh_common")
        .arg("magoosh_adv")
        .output()
        .unwrap();
    info!("command status = {}", output.status);
    let text = output.stdout;
    let response = match size {
        Size::Long => String::from_utf8(text).unwrap(),
        Size::Short => {
            let mut lines = text.lines();
            format!(
                "{}\n{}\n",
                lines.next_line().await.unwrap().unwrap(),
                lines.next_line().await.unwrap().unwrap()
            )
        }
    };
    info!("{}", response);
    response
}
