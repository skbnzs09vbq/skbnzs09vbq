//! `topcoat` CLI エントリーポイント。
//!
//! 本来 `build` サブコマンドの骨格は #11 (SSG ビルドエントリーポイント実装) で作成される
//! 想定だが、issue #15 (feed/sitemap 生成) の動作確認のため、issue #15 のスコープ内で
//! 最小限の `build` サブコマンドを暫定的に用意している。#11 のマージ後は、そちらの
//! ビルドパイプラインに [`topcoat::build::write_feeds_and_sitemap`] の呼び出しを
//! 組み込む形に差し替える。

use std::path::Path;
use std::process::ExitCode;

use topcoat::build::{write_feeds_and_sitemap, SiteData};

/// サイトのベース URL。
///
/// TODO: 設定ファイル/環境変数等、一元管理する仕組みが整備され次第そちらに移行する。
const SITE_BASE_URL: &str = "https://example.com";
const SITE_TITLE: &str = "topcoat";
const SITE_DESCRIPTION: &str = "topcoat works feed";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("build") => run_build(),
        other => {
            eprintln!("unknown command: {other:?} (usage: topcoat build)");
            ExitCode::FAILURE
        }
    }
}

fn run_build() -> ExitCode {
    // TODO(#4, #6, #9, #10, #11): SQLite + Diesel 経由で Work/Tag/Series を取得する処理に
    // 差し替える。現時点ではそれらの前提 issue が未マージのため、空データで動作確認する。
    let data = SiteData::default();

    match write_feeds_and_sitemap(
        Path::new("dist"),
        SITE_BASE_URL,
        SITE_TITLE,
        SITE_DESCRIPTION,
        &data,
    ) {
        Ok(()) => {
            println!("wrote dist/feed.xml, dist/feed.json, dist/sitemap.xml");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("build failed: {err}");
            ExitCode::FAILURE
        }
    }
}
