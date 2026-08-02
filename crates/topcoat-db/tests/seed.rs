//! `seed` バイナリのコア処理（`data`/`insert`/`reset`）の統合テスト。
//!
//! `seed/` 配下のソースは `[[bin]] name = "seed"` 専用のモジュールツリー（ライブラリ
//! 側には公開していない）のため、`#[path]` で同じソースファイルをこのテストバイナリにも
//! 取り込んで再利用する（ロジックを重複実装しない）。
//!
//! `cargo test` はデフォルトで同一バイナリ内のテストを並行実行するため、各テストは
//! `tempfile` で払い出した一意な一時パスへ接続する（固定パスを共有すると SQLite の
//! 書き込みロック競合で間欠的に失敗するため）。

#[path = "../seed/data.rs"]
mod data;
#[path = "../seed/insert.rs"]
mod insert;
#[path = "../seed/reset.rs"]
mod reset;

mod common;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use topcoat_db::schema::{tags, versions, work_tags, works};

/// `reset` → `insert_all` を実行する（`seed` バイナリの無引数実行と同じ処理）。
fn full_seed(conn: &mut SqliteConnection) {
    reset::reset(conn).expect("reset に失敗しました");
    insert::insert_all(conn).expect("insert_all に失敗しました");
}

/// `(tags, works, work_tags, versions)` の件数を取得する。
fn counts(conn: &mut SqliteConnection) -> (i64, i64, i64, i64) {
    let tags_count: i64 = tags::table
        .count()
        .get_result(conn)
        .expect("tags の件数取得に失敗しました");
    let works_count: i64 = works::table
        .count()
        .get_result(conn)
        .expect("works の件数取得に失敗しました");
    let work_tags_count: i64 = work_tags::table
        .count()
        .get_result(conn)
        .expect("work_tags の件数取得に失敗しました");
    let versions_count: i64 = versions::table
        .count()
        .get_result(conn)
        .expect("versions の件数取得に失敗しました");

    (tags_count, works_count, work_tags_count, versions_count)
}

/// `works` の `slug` 一覧を `id` 昇順で取得する（内容比較用）。
fn work_slugs(conn: &mut SqliteConnection) -> Vec<String> {
    works::table
        .select(works::slug)
        .order(works::id.asc())
        .load(conn)
        .expect("works の slug 一覧取得に失敗しました")
}

#[test]
fn seed_inserts_expected_counts() {
    let (_db_file, mut conn) = common::setup_connection();

    full_seed(&mut conn);

    let (tags_count, works_count, work_tags_count, versions_count) = counts(&mut conn);

    assert_eq!(tags_count, data::TAG_SEEDS.len() as i64);
    assert_eq!(works_count, data::works().len() as i64);
    assert_eq!(work_tags_count, data::work_tag_pairs().len() as i64);
    assert_eq!(versions_count, data::version_seeds().len() as i64);

    // issue #8 の想定件数目安（20〜30件）を満たすこと。
    assert!(
        (20..=30).contains(&works_count),
        "works 件数は20〜30件目安のはずですが {works_count} 件でした"
    );
}

#[test]
fn seed_is_idempotent_across_multiple_runs() {
    let (_db_file, mut conn) = common::setup_connection();

    full_seed(&mut conn);
    let first_counts = counts(&mut conn);
    let first_slugs = work_slugs(&mut conn);

    // 2回目の実行（reset → insert）を行っても、件数・内容（slug 一覧）が変わらないこと。
    full_seed(&mut conn);
    let second_counts = counts(&mut conn);
    let second_slugs = work_slugs(&mut conn);

    assert_eq!(
        first_counts, second_counts,
        "2回実行しても各テーブルの件数が変わらないこと"
    );
    assert_eq!(
        first_slugs, second_slugs,
        "2回実行しても works の内容（slug 一覧）が変わらないこと"
    );
}

#[test]
fn reset_only_leaves_all_tables_empty() {
    let (_db_file, mut conn) = common::setup_connection();

    full_seed(&mut conn);
    // insert 後、一度でもデータが入っていたことを確認しておく（reset の効果を確認するため）。
    assert_ne!(counts(&mut conn), (0, 0, 0, 0));

    reset::reset(&mut conn).expect("reset に失敗しました");

    assert_eq!(
        counts(&mut conn),
        (0, 0, 0, 0),
        "reset のみ実行した場合、全テーブルが空になること"
    );
}
