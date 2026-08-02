//! `Tag` モデル・`work_tags` を介した Work⇔Tag 取得関数の疎通確認テスト。
//!
//! `cargo test` はデフォルトで同一バイナリ内のテストを並行実行するため、各テストは
//! `tempfile` で払い出した一意な一時パスへ接続する（固定パスを共有すると SQLite の
//! 書き込みロック競合で間欠的に失敗するため）。

use chrono::NaiveDate;
use diesel::prelude::*;
use topcoat_db::models::work::{generate_slug, Work};
use topcoat_db::models::{NewTag, Tag};
use topcoat_db::queries::tag::{tags_for_work, works_for_tag};
use topcoat_db::schema::{tags, work_tags, works};

mod common;

/// `work_tags` の FOREIGN KEY 制約（`establish_connection_at` が接続確立時に
/// `PRAGMA foreign_keys = ON;` を発行するため強制される）を満たすため、実際の
/// `works` テーブルに参照先レコードを insert する。
///
/// `entries` は `(id, "YYYY-MM-DD")` の組。`created_at` 順のテストのため、
/// 呼び出し側が日付を明示的に指定できるようにしている。
fn seed_works(connection: &mut SqliteConnection, entries: &[(i32, &str)]) {
    for &(id, date_str) in entries {
        let created_at = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .expect("有効な日付です")
            .and_hms_opt(0, 0, 0)
            .expect("有効な時刻です");
        let title = format!("Work {id}");

        diesel::insert_into(works::table)
            .values(&Work {
                id,
                title: title.clone(),
                slug: generate_slug(&title),
                description: None,
                series_id: None,
                created_at,
                updated_at: created_at,
                thumbnail: None,
                params: None,
            })
            .execute(connection)
            .expect("works テーブルへのシードデータ投入に失敗しました");
    }
}

/// Tag を1件挿入し、挿入した Tag の id を返す（`slug` はユニーク制約があるため
/// `Tag::find_by_slug` で確実に対象の1件を引き当てられる）。
fn insert_tag(connection: &mut SqliteConnection, name: &str, slug: &str) -> i32 {
    diesel::insert_into(tags::table)
        .values(&NewTag { name, slug })
        .execute(connection)
        .expect("Tag の挿入に失敗しました");

    Tag::find_by_slug(connection, slug)
        .expect("挿入した Tag の取得に失敗しました")
        .expect("挿入した Tag が見つかりませんでした")
        .id
}

fn link(connection: &mut SqliteConnection, work_id: i32, tag_id: i32) {
    diesel::insert_into(work_tags::table)
        .values((work_tags::work_id.eq(work_id), work_tags::tag_id.eq(tag_id)))
        .execute(connection)
        .expect("work_tags の挿入に失敗しました");
}

#[test]
fn find_by_slug_returns_matching_tag() {
    let (_db_file, mut connection) = common::setup_connection();
    insert_tag(&mut connection, "Rust", "rust");

    let found = Tag::find_by_slug(&mut connection, "rust").expect("Tag の取得に失敗しました");

    let tag = found.expect("Tag が見つかりませんでした");
    assert_eq!(tag.name, "Rust");
    assert_eq!(tag.slug, "rust");
}

#[test]
fn find_by_slug_returns_none_when_not_found() {
    let (_db_file, mut connection) = common::setup_connection();

    let found =
        Tag::find_by_slug(&mut connection, "nonexistent").expect("Tag の取得に失敗しました");

    assert!(found.is_none());
}

#[test]
fn all_ordered_by_name_sorts_by_name_then_tiebreaks_by_id() {
    let (_db_file, mut connection) = common::setup_connection();
    insert_tag(&mut connection, "Zeta", "zeta");
    insert_tag(&mut connection, "Alpha", "alpha-1");
    insert_tag(&mut connection, "Alpha", "alpha-2");

    let all = Tag::all_ordered_by_name(&mut connection).expect("Tag 一覧の取得に失敗しました");

    // 同名 "Alpha" は挿入順（id 昇順）でタイブレークされる。
    let slugs: Vec<&str> = all.iter().map(|t| t.slug.as_str()).collect();
    assert_eq!(slugs, vec!["alpha-1", "alpha-2", "zeta"]);
}

#[test]
fn tags_for_work_returns_tags_ordered_by_name() {
    let (_db_file, mut connection) = common::setup_connection();
    seed_works(&mut connection, &[(1, "2026-01-01")]);
    let zeta = insert_tag(&mut connection, "Zeta", "zeta");
    let alpha = insert_tag(&mut connection, "Alpha", "alpha");
    link(&mut connection, 1, zeta);
    link(&mut connection, 1, alpha);

    let result = tags_for_work(&mut connection, 1).expect("tags_for_work に失敗しました");

    let names: Vec<&str> = result.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, vec!["Alpha", "Zeta"]);
}

#[test]
fn tags_for_work_returns_empty_vec_when_work_has_no_tags() {
    let (_db_file, mut connection) = common::setup_connection();
    seed_works(&mut connection, &[(1, "2026-01-01")]);

    let result = tags_for_work(&mut connection, 1).expect("tags_for_work に失敗しました");

    assert!(result.is_empty());
}

#[test]
fn works_for_tag_returns_multiple_works_ordered_by_created_at() {
    let (_db_file, mut connection) = common::setup_connection();
    // あえて id 順とは異なる created_at 順で作成し、created_at 昇順で
    // ソートされることを検証する。
    seed_works(
        &mut connection,
        &[(1, "2026-01-02"), (2, "2026-01-01"), (3, "2026-01-03")],
    );
    let rust = insert_tag(&mut connection, "Rust", "rust");
    link(&mut connection, 1, rust);
    link(&mut connection, 2, rust);
    link(&mut connection, 3, rust);

    let result = works_for_tag(&mut connection, rust).expect("works_for_tag に失敗しました");

    let ids: Vec<i32> = result.iter().map(|w| w.id).collect();
    assert_eq!(ids, vec![2, 1, 3]);
}

#[test]
fn works_for_tag_excludes_works_without_the_tag() {
    let (_db_file, mut connection) = common::setup_connection();
    seed_works(&mut connection, &[(1, "2026-01-01"), (2, "2026-01-02")]);
    let rust = insert_tag(&mut connection, "Rust", "rust");
    let other = insert_tag(&mut connection, "Other", "other");
    link(&mut connection, 1, rust);
    link(&mut connection, 2, other);

    let result = works_for_tag(&mut connection, rust).expect("works_for_tag に失敗しました");

    let ids: Vec<i32> = result.iter().map(|w| w.id).collect();
    assert_eq!(ids, vec![1]);
}

#[test]
fn works_for_tag_returns_empty_vec_when_tag_has_no_works() {
    let (_db_file, mut connection) = common::setup_connection();
    let tag_id = insert_tag(&mut connection, "Unused", "unused");

    let result = works_for_tag(&mut connection, tag_id).expect("works_for_tag に失敗しました");

    assert!(result.is_empty());
}
