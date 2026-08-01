//! 関連作品算出ロジック（[`topcoat_db::queries::related`]）のテスト。
//!
//! `cargo test` はデフォルトで同一バイナリ内のテストを並行実行するため、各テストは
//! `tempfile` で払い出した一意な一時パスへ接続する（固定パスを共有すると SQLite の
//! 書き込みロック競合で間欠的に失敗するため）。

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use tempfile::NamedTempFile;
use topcoat_db::queries::related::{related_works, RelatedWork};
use topcoat_db::schema::{related_works as related_works_table, tags, work_tags, works};

/// 一時 DB へのマイグレーション適用済みコネクションを払い出す。
///
/// 戻り値の `NamedTempFile` は drop されると DB ファイルごと削除されるため、
/// テスト関数のスコープを抜けるまで呼び出し側で保持しておく必要がある。
fn setup_db() -> (NamedTempFile, SqliteConnection) {
    let db_file = NamedTempFile::new().expect("一時 DB ファイルの作成に失敗しました");
    let mut connection = topcoat_db::establish_connection_at(db_file.path())
        .expect("SQLite への接続確立に失敗しました");
    topcoat_db::run_migrations(&mut connection).expect("マイグレーションの適用に失敗しました");
    (db_file, connection)
}

fn insert_work(conn: &mut SqliteConnection, id: i32, slug: &str, title: &str, thumbnail: &str) {
    diesel::insert_into(works::table)
        .values((
            works::id.eq(id),
            works::slug.eq(slug),
            works::title.eq(title),
            works::thumbnail.eq(thumbnail),
        ))
        .execute(conn)
        .expect("work の insert に失敗しました");
}

fn insert_tag(conn: &mut SqliteConnection, id: i32, slug: &str, name: &str) {
    diesel::insert_into(tags::table)
        .values((tags::id.eq(id), tags::slug.eq(slug), tags::name.eq(name)))
        .execute(conn)
        .expect("tag の insert に失敗しました");
}

fn insert_work_tag(conn: &mut SqliteConnection, work_id: i32, tag_id: i32) {
    diesel::insert_into(work_tags::table)
        .values((work_tags::work_id.eq(work_id), work_tags::tag_id.eq(tag_id)))
        .execute(conn)
        .expect("work_tags の insert に失敗しました");
}

fn insert_related_work(conn: &mut SqliteConnection, work_id: i32, related_work_id: i32) {
    diesel::insert_into(related_works_table::table)
        .values((
            related_works_table::work_id.eq(work_id),
            related_works_table::related_work_id.eq(related_work_id),
        ))
        .execute(conn)
        .expect("related_works の insert に失敗しました");
}

fn slugs(works: &[RelatedWork]) -> Vec<&str> {
    works.iter().map(|w| w.slug.as_str()).collect()
}

/// `related_works` テーブルに明示的リレーションが1件でも存在する場合、
/// 共有タグ数がより多い他の Work があっても明示的リレーションを優先して返すこと。
#[test]
fn explicit_relation_takes_priority_over_shared_tag_fallback() {
    let (_db_file, mut conn) = setup_db();

    insert_work(&mut conn, 1, "work-1", "Work 1", "thumb-1");
    insert_work(&mut conn, 2, "work-2", "Work 2", "thumb-2");
    insert_work(&mut conn, 3, "work-3", "Work 3", "thumb-3");

    insert_tag(&mut conn, 1, "tag-a", "Tag A");
    insert_tag(&mut conn, 2, "tag-b", "Tag B");

    // work-2 は work-1 と全タグを共有する（フォールバックなら1位になるはず）が、
    // work-1 → work-3 の明示的リレーションが登録されているため、work-3 が優先される。
    insert_work_tag(&mut conn, 1, 1);
    insert_work_tag(&mut conn, 1, 2);
    insert_work_tag(&mut conn, 2, 1);
    insert_work_tag(&mut conn, 2, 2);

    insert_related_work(&mut conn, 1, 3);

    let result = related_works(&mut conn, 1, None).expect("related_works の取得に失敗しました");

    assert_eq!(slugs(&result), vec!["work-3"]);
    // 明示的リレーションでも shared_tag_count は実際の共有タグ数を計算して埋める。
    // work-1 と work-3 の共有タグは0件。
    assert_eq!(result[0].shared_tag_count, 0);
}

/// 明示的リレーションが存在しない場合、共有タグ数（`work_tags` の積集合サイズ）が
/// 多い順にソートされ、`limit` 件数で打ち切られること。
#[test]
fn fallback_sorts_by_shared_tag_count_desc_and_respects_limit() {
    let (_db_file, mut conn) = setup_db();

    insert_work(&mut conn, 1, "target", "Target", "thumb-target");
    insert_work(&mut conn, 2, "four-shared", "Four Shared", "thumb-2");
    insert_work(&mut conn, 3, "three-shared", "Three Shared", "thumb-3");
    insert_work(&mut conn, 4, "two-shared", "Two Shared", "thumb-4");
    insert_work(&mut conn, 5, "one-shared", "One Shared", "thumb-5");

    for tag_id in 1..=4 {
        insert_tag(
            &mut conn,
            tag_id,
            &format!("tag-{tag_id}"),
            &format!("Tag {tag_id}"),
        );
        insert_work_tag(&mut conn, 1, tag_id);
    }

    // work-2: t1,t2,t3,t4 を共有（4件）
    for tag_id in 1..=4 {
        insert_work_tag(&mut conn, 2, tag_id);
    }
    // work-3: t1,t2,t3 を共有（3件）
    for tag_id in 1..=3 {
        insert_work_tag(&mut conn, 3, tag_id);
    }
    // work-4: t1,t2 を共有（2件）
    for tag_id in 1..=2 {
        insert_work_tag(&mut conn, 4, tag_id);
    }
    // work-5: t1 を共有（1件）
    insert_work_tag(&mut conn, 5, 1);

    // limit=3 を明示指定した場合、上位3件（work-2, work-3, work-4）のみ返る。
    let limited = related_works(&mut conn, 1, Some(3)).expect("related_works の取得に失敗しました");
    assert_eq!(
        slugs(&limited),
        vec!["four-shared", "three-shared", "two-shared"]
    );
    assert_eq!(
        limited
            .iter()
            .map(|w| w.shared_tag_count)
            .collect::<Vec<_>>(),
        vec![4, 3, 2]
    );

    // limit 未指定時はデフォルト（4件）まで返り、work-5 も含まれる。
    let default_limit =
        related_works(&mut conn, 1, None).expect("related_works の取得に失敗しました");
    assert_eq!(
        slugs(&default_limit),
        vec!["four-shared", "three-shared", "two-shared", "one-shared"]
    );
}

/// フォールバック算出時、対象 Work 自身は結果から除外されること。
#[test]
fn fallback_excludes_the_target_work_itself() {
    let (_db_file, mut conn) = setup_db();

    insert_work(&mut conn, 1, "target", "Target", "thumb-target");
    insert_work(&mut conn, 2, "other", "Other", "thumb-other");

    insert_tag(&mut conn, 1, "tag-a", "Tag A");
    insert_work_tag(&mut conn, 1, 1);
    insert_work_tag(&mut conn, 2, 1);

    let result = related_works(&mut conn, 1, None).expect("related_works の取得に失敗しました");

    // work-1（自分自身）は含まれず、work-2 のみが返る。
    assert_eq!(slugs(&result), vec!["other"]);
}
