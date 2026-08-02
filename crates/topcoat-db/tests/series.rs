//! `Series` モデルの疎通テスト。
//!
//! `cargo test` はデフォルトで同一バイナリ内のテストを並行実行するため、各テストは
//! `tempfile` で払い出した一意な一時パスへ接続する（固定パスを共有すると SQLite の
//! 書き込みロック競合で間欠的に失敗するため）。

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use tempfile::NamedTempFile;
use topcoat_db::models::{NewSeries, Series};
use topcoat_db::schema::series;

/// マイグレーション適用済みの一時 DB への接続を確立する。
///
/// 戻り値の `NamedTempFile` は呼び出し側で保持し続けること（drop されると
/// DB ファイルごと削除されてしまう）。
fn setup_connection() -> (NamedTempFile, SqliteConnection) {
    let db_file = NamedTempFile::new().expect("一時 DB ファイルの作成に失敗しました");

    let mut connection = topcoat_db::establish_connection_at(db_file.path())
        .expect("SQLite への接続確立に失敗しました");
    topcoat_db::run_migrations(&mut connection).expect("マイグレーションの適用に失敗しました");

    (db_file, connection)
}

fn insert_series(connection: &mut SqliteConnection, name: &str, slug: &str, description: &str) {
    diesel::insert_into(series::table)
        .values(NewSeries {
            name,
            slug,
            description,
        })
        .execute(connection)
        .expect("Series の挿入に失敗しました");
}

#[test]
fn find_by_slug_returns_matching_series() {
    let (_db_file, mut connection) = setup_connection();

    insert_series(&mut connection, "テストシリーズ", "test-series", "説明文");

    let found = Series::find_by_slug(&mut connection, "test-series")
        .expect("クエリの実行に失敗しました")
        .expect("Series が見つかりませんでした");

    assert_eq!(found.name, "テストシリーズ");
    assert_eq!(found.slug, "test-series");
    assert_eq!(found.description, "説明文");
}

#[test]
fn find_by_slug_returns_none_when_not_found() {
    let (_db_file, mut connection) = setup_connection();

    let found = Series::find_by_slug(&mut connection, "nonexistent-slug")
        .expect("クエリの実行に失敗しました");

    assert!(found.is_none());
}

#[test]
fn all_ordered_by_name_returns_series_sorted_by_name() {
    let (_db_file, mut connection) = setup_connection();

    insert_series(&mut connection, "Bシリーズ", "b-series", "");
    insert_series(&mut connection, "Aシリーズ", "a-series", "");
    insert_series(&mut connection, "Cシリーズ", "c-series", "");

    let all = Series::all_ordered_by_name(&mut connection).expect("クエリの実行に失敗しました");
    let names: Vec<&str> = all.iter().map(|s| s.name.as_str()).collect();

    assert_eq!(names, vec!["Aシリーズ", "Bシリーズ", "Cシリーズ"]);
}
