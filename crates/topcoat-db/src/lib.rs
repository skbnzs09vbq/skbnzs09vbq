//! `topcoat-db` — `topcoat` の DB スキーマ・エンティティ・ORM アクセスを担うライブラリ crate。
//!
//! 本 issue（#4: SQLite + Diesel ORM 導入とマイグレーション基盤構築）では、SQLite
//! （ファイルベース、ビルド時のみ読み書き）への接続確立と `diesel_migrations` による
//! マイグレーション自動適用の疎通確認までをスコープとする。
//! Work/Tag 等の実テーブル定義・エンティティ実装は後続 issue（#6 等）で行う。

pub mod schema;

use std::fs;
use std::path::{Path, PathBuf};

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
fn database_path() -> PathBuf {
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

    SqliteConnection::establish(&db_path.to_string_lossy())
}

/// 埋め込まれたマイグレーション（`MIGRATIONS`）を DB に適用する。
///
/// 未適用のマイグレーションのみが適用されるため、複数回呼び出しても安全（冪等）。
/// 本 issue時点ではマイグレーションファイル自体が存在しない（テーブル未定義）ため、
/// 実質的には「マイグレーション管理テーブルの初期化のみ行われて即座に成功する」動作になる。
pub fn run_migrations(
    connection: &mut SqliteConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    connection.run_pending_migrations(MIGRATIONS).map(|_| ())
}
