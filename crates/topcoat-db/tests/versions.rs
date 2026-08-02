//! `Version` モデル・`Version::for_work` クエリの疎通確認テスト。
//!
//! `cargo test` はデフォルトで同一バイナリ内のテストを並行実行するため、各テストは
//! `tempfile` で払い出した一意な一時パスへ接続する（固定パスを共有すると SQLite の
//! 書き込みロック競合で間欠的に失敗するため）。

use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use tempfile::NamedTempFile;
use topcoat_db::models::{NewVersion, Version};
use topcoat_db::schema::versions;

/// `works` テーブルは別 issue（#6）で追加される。`topcoat_db::establish_connection_at`
/// が接続確立時に `PRAGMA foreign_keys = ON;` を発行するため、`versions.work_id` の
/// FOREIGN KEY 制約を満たすテスト用の最小限のスタブテーブルを用意する。
fn seed_stub_works_table(connection: &mut SqliteConnection) {
    connection
        .batch_execute(
            "CREATE TABLE works (id INTEGER NOT NULL PRIMARY KEY);
             INSERT INTO works (id) VALUES (1), (2);",
        )
        .expect("works テーブル（スタブ）の準備に失敗しました");
}

#[test]
fn for_work_returns_only_matching_work_id_in_chronological_order() {
    let db_file = NamedTempFile::new().expect("一時 DB ファイルの作成に失敗しました");

    let mut connection = topcoat_db::establish_connection_at(db_file.path())
        .expect("SQLite への接続確立に失敗しました");
    topcoat_db::run_migrations(&mut connection).expect("マイグレーションの適用に失敗しました");
    seed_stub_works_table(&mut connection);

    // work_id=1 に対し、あえて "v2" → "v1" の順で挿入する。
    diesel::insert_into(versions::table)
        .values(&NewVersion {
            work_id: 1,
            version_label: "v2",
            changelog: "2回目の変更",
        })
        .execute(&mut connection)
        .expect("Version の挿入に失敗しました");

    diesel::insert_into(versions::table)
        .values(&NewVersion {
            work_id: 1,
            version_label: "v1",
            changelog: "初回リリース",
        })
        .execute(&mut connection)
        .expect("Version の挿入に失敗しました");

    // 別 work_id のレコードは結果に含まれないことを確認するためのノイズデータ。
    diesel::insert_into(versions::table)
        .values(&NewVersion {
            work_id: 2,
            version_label: "v1",
            changelog: "別作品の初回リリース",
        })
        .execute(&mut connection)
        .expect("Version の挿入に失敗しました");

    let result = Version::for_work(&mut connection, 1).expect("Version の取得に失敗しました");

    // work_id=1 の2件のみが、挿入順（id 昇順。created_at が同秒になり得るための副次キー）で返る。
    let labels: Vec<&str> = result
        .iter()
        .map(|version| version.version_label.as_str())
        .collect();
    assert_eq!(labels, vec!["v2", "v1"]);
    assert!(result.iter().all(|version| version.work_id == 1));
    assert!(result.iter().all(|version| !version.changelog.is_empty()));
}

#[test]
fn for_work_returns_empty_vec_when_no_versions_exist() {
    let db_file = NamedTempFile::new().expect("一時 DB ファイルの作成に失敗しました");

    let mut connection = topcoat_db::establish_connection_at(db_file.path())
        .expect("SQLite への接続確立に失敗しました");
    topcoat_db::run_migrations(&mut connection).expect("マイグレーションの適用に失敗しました");

    let result = Version::for_work(&mut connection, 999).expect("Version の取得に失敗しました");

    assert!(result.is_empty());
}
