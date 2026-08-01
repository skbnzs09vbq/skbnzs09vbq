//! `topcoat` — 自作 SSG (Static Site Generator) の CLI エントリーポイント。
//!
//! `build` サブコマンドの骨格は #11 (SSG ビルドエントリーポイント実装) で作成される想定だが、
//! issue #15 (feed/sitemap 生成)・#12 (作品詳細静的ページ生成) の動作確認のため、それぞれの
//! スコープ内で最小限の `build` サブコマンドを暫定的に用意している。#11 のマージ後は、
//! そちらのビルドパイプラインに [`topcoat::build::write_feeds_and_sitemap`] /
//! [`topcoat::build::write_work_detail_pages`] の呼び出しを組み込む形に差し替える。

use std::path::Path;
use std::process::ExitCode;

use topcoat::build::{write_feeds_and_sitemap, write_work_detail_pages, SiteData};

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
    let dist_dir = Path::new("dist");

    if let Err(err) =
        write_feeds_and_sitemap(dist_dir, SITE_BASE_URL, SITE_TITLE, SITE_DESCRIPTION, &data)
    {
        eprintln!("build failed: {err}");
        return ExitCode::FAILURE;
    }
    println!("wrote dist/feed.xml, dist/feed.json, dist/sitemap.xml");

    let tera = match topcoat_render::build_tera() {
        Ok(tera) => tera,
        Err(err) => {
            eprintln!("build failed: failed to load templates: {err}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(err) = write_work_detail_pages(dist_dir, &tera, &data.works) {
        eprintln!("build failed: {err}");
        return ExitCode::FAILURE;
    }
    println!(
        "wrote dist/works/<slug>/index.html for {} work(s)",
        data.works.len()
    );

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
