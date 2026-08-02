//! `topcoat-db` — `topcoat` の DB スキーマ・エンティティ・ORM アクセスを担うライブラリ crate。
//!
//! issue #4（SQLite + Diesel ORM 導入とマイグレーション基盤構築）で、SQLite
//! （ファイルベース、ビルド時のみ読み書き）への接続確立と `diesel_migrations` による
//! マイグレーション自動適用の基盤を構築した。
//! issue #6 で `works` テーブルと [`models::work::Work`] エンティティを追加している。
//!
//! `tags` / `work_tags` / `related_works` テーブルは issue #7（関連作品算出ロジック実装）が
//! 追加したもの。issue #7 着手時点では #6 が未マージだったため `works` テーブル自体も
//! 暫定スキーマとして先行作成されていたが、#6 マージ時に issue #6 が正式に定義する完全な
//! `works` スキーマへ統合済み（詳細は `migrations/2026-08-01-170200-0000_create_works` の
//! コメントおよび [`queries::related`] を参照）。
//! `series` テーブルと [`models::Series`] エンティティは issue #1 で追加している。

pub mod models;
pub mod queries;
pub mod schema;

use std::fs;
use std::path::{Path, PathBuf};

use diesel::connection::SimpleConnection;
use diesel::sqlite::SqliteConnection;
use diesel::{Connection, ConnectionError, ConnectionResult};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

/// `migrations/` ディレクトリの中身をバイナリに埋め込んだもの。
///
/// `embed_migrations!` のパスは `CARGO_MANIFEST_DIR`（= `crates/topcoat-db`）基準で解決される。
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// DB ファイルの配置先パスを解決する。
///
/// `CARGO_MANIFEST_DIR` 基準で `data/site.sqlite3` を返すため、
/// 実行時のカレントディレクトリ（CWD）に依存しない。
///
/// 本番用の固定パスが必要な呼び出し元（`topcoat` の `main.rs` 等）が
/// [`establish_connection_at`] に渡すために利用する。テストでは代わりに
/// `tempfile` 等で払い出した一時パスを使うこと。
pub fn database_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("site.sqlite3")
}

/// SQLite への接続を確立する。
///
/// DB ファイルの配置先ディレクトリ（`data/`）が存在しない場合は先に作成する。
pub fn establish_connection() -> ConnectionResult<SqliteConnection> {
    establish_connection_at(&database_path())
}

/// 指定した任意のパスの SQLite ファイルへの接続を確立する。
///
/// `db_path` の配置先ディレクトリが存在しない場合は先に作成する。
/// テストなど、固定パス（[`establish_connection`]）とは別の DB ファイルを
/// 使いたい場合に利用する。
///
/// SQLite は接続ごとに `PRAGMA foreign_keys` が既定で無効になっているため、
/// 確立した接続に対して明示的に `PRAGMA foreign_keys = ON;` を発行し、
/// FOREIGN KEY 制約（例: `versions.work_id`）がランタイムで強制されるようにする。
pub fn establish_connection_at(db_path: &Path) -> ConnectionResult<SqliteConnection> {
    if let Some(dir) = db_path.parent() {
        if !dir.exists() {
            fs::create_dir_all(dir).map_err(|err| {
                ConnectionError::BadConnection(format!(
                    "DB 配置先ディレクトリの作成に失敗しました ({}): {err}",
                    dir.display()
                ))
            })?;
        }
    }

    let mut connection = SqliteConnection::establish(&db_path.to_string_lossy())?;
    connection
        .batch_execute("PRAGMA foreign_keys = ON;")
        .map_err(ConnectionError::CouldntSetupConfiguration)?;

    Ok(connection)
}

/// 埋め込まれたマイグレーション（`MIGRATIONS`）を DB に適用する。
///
/// 未適用のマイグレーションのみが適用されるため、複数回呼び出しても安全（冪等）。
/// `works` / `tags` / `work_tags` / `related_works` の4テーブルを作成するマイグレーションが
/// 含まれる（詳細はモジュール冒頭のドキュメントおよび各マイグレーションのコメントを参照）。
pub fn run_migrations(
    connection: &mut SqliteConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    connection.run_pending_migrations(MIGRATIONS).map(|_| ())
}
