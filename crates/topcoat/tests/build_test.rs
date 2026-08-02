//! `topcoat build` パイプライン (`topcoat::build::run`) の統合テスト。
//!
//! マイグレーション適用・(プレースホルダの) データ取得・レンダリングを経て、
//! 空データでも `dist/index.html` を含む成果物一式が正常に出力されることを検証する。
//!
//! `BuildConfig::db_path` に `tempfile` で払い出した一時パスを渡すことで、本番用の
//! 固定パス SQLite ファイル (`topcoat-db/data/site.sqlite3`) を汚染しないようにしている。

use std::path::Path;

use topcoat::build::{run, BuildConfig};

fn config(db_path: &Path) -> BuildConfig {
    BuildConfig {
        base_url: "https://example.com".to_string(),
        site_title: "Test Site".to_string(),
        site_description: "desc".to_string(),
        db_path: db_path.to_path_buf(),
    }
}

#[test]
fn run_with_empty_data_writes_expected_output_and_is_idempotent() {
    let dist_dir = tempfile::tempdir().expect("一時ディレクトリの作成に失敗しました");
    let db_file = tempfile::NamedTempFile::new().expect("一時 DB ファイルの作成に失敗しました");

    run(dist_dir.path(), &config(db_file.path())).expect("1回目の build::run が失敗しました");

    assert!(dist_dir.path().join("index.html").is_file());
    assert!(dist_dir.path().join("feed.xml").is_file());
    assert!(dist_dir.path().join("feed.json").is_file());
    assert!(dist_dir.path().join("sitemap.xml").is_file());
    assert!(dist_dir.path().join("assets/css/tokens.css").is_file());

    let index_html = std::fs::read_to_string(dist_dir.path().join("index.html"))
        .expect("dist/index.html の読み込みに失敗しました");
    assert!(index_html.contains("<!DOCTYPE html>"));
    assert!(index_html.contains("<title>topcoat</title>"));

    // 同一 DB ファイルに対して複数回実行しても (マイグレーション適用・ファイル書き出しとも)
    // 正常終了すること。
    run(dist_dir.path(), &config(db_file.path())).expect("2回目の build::run が失敗しました");
    assert!(dist_dir.path().join("index.html").is_file());
}
