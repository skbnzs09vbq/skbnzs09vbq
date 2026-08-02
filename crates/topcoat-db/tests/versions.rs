//! `Version` モデル・`Version::for_work` クエリの疎通確認テスト。
//!
//! `cargo test` はデフォルトで同一バイナリ内のテストを並行実行するため、各テストは
//! `tempfile` で払い出した一意な一時パスへ接続する（固定パスを共有すると SQLite の
//! 書き込みロック競合で間欠的に失敗するため）。

use chrono::NaiveDate;
use diesel::prelude::*;
use topcoat_db::models::work::{generate_slug, Work};
use topcoat_db::models::{NewVersion, Version};
use topcoat_db::schema::{versions, works};

mod common;

/// `versions.work_id` の FOREIGN KEY 制約（`establish_connection_at` が接続確立時に
/// `PRAGMA foreign_keys = ON;` を発行するため強制される）を満たすため、実際の
/// `works` テーブルに参照先レコードを insert する。
fn seed_works(connection: &mut SqliteConnection, ids: &[i32]) {
    let now = NaiveDate::from_ymd_opt(2026, 1, 1)
        .expect("有効な日付です")
        .and_hms_opt(0, 0, 0)
        .expect("有効な時刻です");

    for &id in ids {
        let title = format!("Work {id}");
        diesel::insert_into(works::table)
            .values(&Work {
                id,
                title: title.clone(),
                slug: generate_slug(&title),
                description: None,
                series_id: None,
                created_at: now,
                updated_at: now,
                thumbnail: None,
                params: None,
            })
            .execute(connection)
            .expect("works テーブルへのシードデータ投入に失敗しました");
    }
}

#[test]
fn for_work_returns_only_matching_work_id_in_chronological_order() {
    let (_db_file, mut connection) = common::setup_connection();
    seed_works(&mut connection, &[1, 2]);

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
    let (_db_file, mut connection) = common::setup_connection();

    let result = Version::for_work(&mut connection, 999).expect("Version の取得に失敗しました");

    assert!(result.is_empty());
}
