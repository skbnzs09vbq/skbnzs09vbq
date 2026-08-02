//! `work_tags` 中間テーブルを介した Work⇔Tag の取得関数。
//!
//! [`crate::queries::related`] の JOIN パターン（対象テーブルへの `inner_join` +
//! `filter` + `select`）に倣う。

use diesel::prelude::*;

use crate::models::work::Work;
use crate::models::Tag;
use crate::schema::{tags, work_tags, works};

/// 指定した `work_id` に紐づく Tag 一覧を、`name` 昇順（同名時は `id` 昇順でタイブレーク）
/// で取得する。
pub fn tags_for_work(conn: &mut SqliteConnection, work_id: i32) -> QueryResult<Vec<Tag>> {
    work_tags::table
        .inner_join(tags::table.on(tags::id.eq(work_tags::tag_id)))
        .filter(work_tags::work_id.eq(work_id))
        .order((tags::name.asc(), tags::id.asc()))
        .select(Tag::as_select())
        .load::<Tag>(conn)
}

/// 指定した `tag_id` に紐づく Work 一覧を、`created_at` 昇順（同時刻は `id` 昇順で
/// タイブレーク）で取得する。
pub fn works_for_tag(conn: &mut SqliteConnection, tag_id: i32) -> QueryResult<Vec<Work>> {
    work_tags::table
        .inner_join(works::table.on(works::id.eq(work_tags::work_id)))
        .filter(work_tags::tag_id.eq(tag_id))
        .order((works::created_at.asc(), works::id.asc()))
        .select(Work::as_select())
        .load::<Work>(conn)
}
