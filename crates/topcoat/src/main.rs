//! `topcoat` — 自作 SSG (Static Site Generator) の CLI エントリーポイント。

use std::path::Path;
use std::process::ExitCode;

use topcoat::build::{self, BuildConfig};

/// サイトのベース URL。
///
/// TODO: 設定ファイル/環境変数等、一元管理する仕組みが整備され次第そちらに移行する。
const SITE_BASE_URL: &str = "https://example.com";
const SITE_TITLE: &str = "topcoat";
const SITE_DESCRIPTION: &str = "topcoat works feed";

/// `topcoat` が受け付けるサブコマンド。
enum Command {
    /// サイトをビルドする。
    Build,
}

fn parse_command(mut args: impl Iterator<Item = String>) -> Result<Command, String> {
    // args[0] はバイナリ名のため読み飛ばす。
    args.next();

    match args.next().as_deref() {
        Some("build") => Ok(Command::Build),
        Some(other) => Err(format!("unknown subcommand: {other}")),
        None => Err("missing subcommand (expected: build)".to_string()),
    }
}

fn run(command: Command) -> ExitCode {
    match command {
        Command::Build => run_build(),
    }
}

fn run_build() -> ExitCode {
    let config = BuildConfig {
        base_url: SITE_BASE_URL.to_string(),
        site_title: SITE_TITLE.to_string(),
        site_description: SITE_DESCRIPTION.to_string(),
        db_path: topcoat_db::database_path(),
    };

    match build::run(Path::new("dist"), &config) {
        Ok(()) => {
            println!(
                "wrote dist/index.html, dist/feed.xml, dist/feed.json, dist/sitemap.xml, dist/works/<slug>/og.png"
            );
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("build failed: {err}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    match parse_command(std::env::args()) {
        Ok(command) => run(command),
        Err(message) => {
            eprintln!("error: {message}");
            eprintln!("usage: topcoat build");
            ExitCode::FAILURE
        }
    }
}
