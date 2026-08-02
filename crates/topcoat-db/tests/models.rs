//! `Work` エンティティの insert/select 往復と `slug` の UNIQUE 制約違反を確認するテスト。
//!
//! `cargo test` はデフォルトで同一バイナリ内のテストを並行実行するため、各テストは
//! `tempfile` で払い出した一意な一時パスへ接続する（固定パスを共有すると SQLite の
//! 書き込みロック競合で間欠的に失敗するため）。

use chrono::NaiveDate;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use topcoat_db::models::work::{generate_slug, Work};
use topcoat_db::schema::works;

mod common;

fn sample_work(id: i32, title: &str) -> Work {
    let now = NaiveDate::from_ymd_opt(2026, 1, 1)
        .expect("有効な日付です")
        .and_hms_opt(0, 0, 0)
        .expect("有効な時刻です");

    Work {
        id,
        title: title.to_string(),
        slug: generate_slug(title),
        description: Some("説明文".to_string()),
        series_id: None,
        created_at: now,
        updated_at: now,
        thumbnail: None,
        params: None,
    }
}

#[test]
fn insert_and_select_work_round_trip() {
    let (_db_file, mut connection) = common::setup_connection();

    let work = sample_work(1, "Hello World");

    diesel::insert_into(works::table)
        .values(&work)
        .execute(&mut connection)
        .expect("Work の insert に失敗しました");

    let found: Work = works::table
        .filter(works::id.eq(1))
        .select(Work::as_select())
        .first(&mut connection)
        .expect("Work の select に失敗しました");

    assert_eq!(found, work);
    assert_eq!(found.slug, "hello-world");
}

#[test]
fn inserting_duplicate_slug_violates_unique_constraint() {
    let (_db_file, mut connection) = common::setup_connection();

    let first = sample_work(1, "Hello World");
    diesel::insert_into(works::table)
        .values(&first)
        .execute(&mut connection)
        .expect("1件目の Work の insert に失敗しました");

    // タイトルは異なるが slug は同一になるレコードを insert し、UNIQUE 制約違反を狙う。
    let mut second = sample_work(2, "Hello World");
    second.slug = first.slug.clone();

    let result = diesel::insert_into(works::table)
        .values(&second)
        .execute(&mut connection);

    match result {
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {}
        other => panic!("UNIQUE 制約違反を期待しましたが、結果は {other:?} でした"),
    }
}
