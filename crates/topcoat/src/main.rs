//! `topcoat` — 自作 SSG (Static Site Generator) の CLI エントリーポイント。
//!
//! `build` サブコマンドの骨格は #11 (SSG ビルドエントリーポイント実装) で作成される想定だが、
//! issue #15 (feed/sitemap 生成)・#13 (OG 画像生成) の動作確認のため、それぞれのスコープ内で
//! 最小限の `build` サブコマンドを暫定的に用意している。#11 のマージ後は、そちらのビルド
//! パイプラインに [`topcoat::build::write_feeds_and_sitemap`] /
//! [`topcoat::build::write_og_images`] の呼び出しを組み込む形に差し替える。

use std::path::Path;
use std::process::ExitCode;

use topcoat::build::{write_feeds_and_sitemap, write_og_images, SiteData};

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
    // TODO(#4, #6, #9, #10, #11): SQLite + Diesel 経由で Work/Tag/Series を取得する処理に
    // 差し替える。現時点ではそれらの前提 issue が未マージのため、空データで動作確認する。
    let data = SiteData::default();

    if let Err(err) = write_feeds_and_sitemap(
        Path::new("dist"),
        SITE_BASE_URL,
        SITE_TITLE,
        SITE_DESCRIPTION,
        &data,
    ) {
        eprintln!("build failed: {err}");
        return ExitCode::FAILURE;
    }
    println!("wrote dist/feed.xml, dist/feed.json, dist/sitemap.xml");

    if let Err(err) = write_og_images(Path::new("dist"), &data) {
        eprintln!("build failed: {err}");
        return ExitCode::FAILURE;
    }
    println!("wrote dist/works/<slug>/og.png");

    ExitCode::SUCCESS
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
