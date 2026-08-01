//! Work の「関連作品」算出ロジック。
//!
//! `related_works` テーブルによる明示的リレーションを優先する。1件も登録されていない
//! 場合は、`work_tags` を介した共有タグ数（積集合サイズ）が多い順に上位N件を自動算出する
//! フォールバックで求める。
//!
//! 暫定実装に関する注記: 本モジュールが依存する `works` / `tags` / `work_tags` テーブルは、
//! 本来 issue #6（Workエンティティ）・#2（Tagエンティティと Work-Tag 多対多リレーション）で
//! 定義される想定だが、issue #7 着手時点でいずれも未マージのため、`crates/topcoat-db/migrations`
//! 配下に issue #7 が必要とする範囲に限定した暫定スキーマとして先行作成している
//! （詳細は各マイグレーションのコメントを参照）。#6・#2 マージ後は、カラム構成の差分を
//! 追加マイグレーションで揃えること。

use std::collections::HashMap;

use diesel::dsl::count;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::QueryResult;
use serde::Serialize;

use crate::schema::{related_works, work_tags, works};

/// フォールバック（共有タグ数算出）で返す関連作品件数のデフォルト値。
pub const DEFAULT_RELATED_WORKS_LIMIT: usize = 4;

/// 関連作品1件分の情報。
///
/// 明示的リレーション経由・共有タグ数フォールバック経由のどちらで取得した場合も、
/// 呼び出し側（ビルド処理・表示側）がソースを区別せず統一的に扱えるよう、この型に
/// 統一して返す。`shared_tag_count` は明示的リレーション経由の場合も実際の共有タグ数を
/// 計算して埋める（0固定にはしない）。
#[derive(Debug, Clone, Serialize)]
pub struct RelatedWork {
    pub slug: String,
    pub title: String,
    pub thumbnail: String,
    pub shared_tag_count: i64,
}

/// `(id, slug, title, thumbnail)` — Work 1件分の基本情報。
type WorkRow = (i32, String, String, String);

/// `(id, slug, title, thumbnail, shared_tag_count)` — 共有タグ数付きの Work 情報。
type RankedWorkRow = (i32, String, String, String, i64);

/// `related_works` テーブルに登録された、`work_id` の明示的リレーション先を取得する。
///
/// `related_work_id` 昇順（登録順）で返し、件数の上限は設けない
/// （明示的に登録された分はすべて関連作品として扱う）。
fn explicit_related_works(conn: &mut SqliteConnection, work_id: i32) -> QueryResult<Vec<WorkRow>> {
    related_works::table
        .inner_join(works::table.on(works::id.eq(related_works::related_work_id)))
        .filter(related_works::work_id.eq(work_id))
        .order(related_works::related_work_id.asc())
        .select((works::id, works::slug, works::title, works::thumbnail))
        .load::<(i32, String, String, String)>(conn)
}

/// 対象 `work_id` に紐づく `tag_id` 一覧を取得する。
///
/// `shared_tag_counts_between_many` と `shared_tag_fallback` の両方が、共有タグ数算出の
/// 起点として同じ「対象 Work のタグID一覧取得」を必要とするため、共通ヘルパーとして
/// 切り出している（空判定・早期リターンは呼び出し側でそのまま行う）。
fn work_tag_ids(conn: &mut SqliteConnection, work_id: i32) -> QueryResult<Vec<i32>> {
    work_tags::table
        .filter(work_tags::work_id.eq(work_id))
        .select(work_tags::tag_id)
        .load(conn)
}

/// `work_id` と `related_ids` それぞれとの間の共有タグ数（`work_tags` の積集合サイズ）を
/// 1クエリで一括計算する。
///
/// 戻り値は `related_id -> shared_tag_count` のマップで、共有タグが0件の
/// `related_id` はキー自体が存在しない（呼び出し側で `unwrap_or(0)` 相当の扱いをする）。
/// `related_ids` ごとに個別クエリを発行する N+1 を避けるため、`work_tags` を
/// `related_ids` 側に対して `eq_any` で一括取得し、`work_id` 単位で件数を集計する。
fn shared_tag_counts_between_many(
    conn: &mut SqliteConnection,
    work_id: i32,
    related_ids: &[i32],
) -> QueryResult<HashMap<i32, i64>> {
    if related_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let tag_ids: Vec<i32> = work_tag_ids(conn, work_id)?;

    if tag_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let counts: Vec<(i32, i64)> = work_tags::table
        .filter(work_tags::work_id.eq_any(related_ids.to_vec()))
        .filter(work_tags::tag_id.eq_any(tag_ids))
        .group_by(work_tags::work_id)
        .select((work_tags::work_id, count(work_tags::tag_id)))
        .load(conn)?;

    Ok(counts.into_iter().collect())
}

/// `work_id` の Work について、`work_tags` を介した共有タグ数で上位 `limit` 件を算出する。
///
/// 対象 Work 自身は除外する。`shared_tag_count DESC` を主ソート、`work_id ASC` を
/// タイブレークとして安定した順序で返す。
fn shared_tag_fallback(
    conn: &mut SqliteConnection,
    work_id: i32,
    limit: i64,
) -> QueryResult<Vec<RankedWorkRow>> {
    let tag_ids: Vec<i32> = work_tag_ids(conn, work_id)?;

    if tag_ids.is_empty() {
        return Ok(Vec::new());
    }

    let ranked: Vec<(i32, i64)> = work_tags::table
        .filter(work_tags::tag_id.eq_any(tag_ids))
        .filter(work_tags::work_id.ne(work_id))
        .group_by(work_tags::work_id)
        .select((work_tags::work_id, count(work_tags::tag_id)))
        .order((count(work_tags::tag_id).desc(), work_tags::work_id.asc()))
        .limit(limit)
        .load::<(i32, i64)>(conn)?;

    if ranked.is_empty() {
        return Ok(Vec::new());
    }

    // ranked の各行ごとに works へ個別クエリすると N+1 になるため、
    // ランク付け後の id 一覧で `eq_any` を使い1クエリで一括取得する。
    let ids: Vec<i32> = ranked.iter().map(|(id, _)| *id).collect();
    let mut rows_by_id: HashMap<i32, WorkRow> = works::table
        .filter(works::id.eq_any(ids))
        .select((works::id, works::slug, works::title, works::thumbnail))
        .load::<WorkRow>(conn)?
        .into_iter()
        .map(|row| (row.0, row))
        .collect();

    ranked
        .into_iter()
        .map(|(related_id, shared_tag_count)| {
            let (_, slug, title, thumbnail) = rows_by_id
                .remove(&related_id)
                .ok_or(diesel::result::Error::NotFound)?;
            Ok((related_id, slug, title, thumbnail, shared_tag_count))
        })
        .collect()
}

/// `work_id` の Work の関連作品を取得する。
///
/// 1. `related_works` テーブルに明示的リレーションが1件でも存在すれば、それを返す
///    （`shared_tag_count` は実際の共有タグ数を計算して埋める）
/// 2. 存在しなければ、共有タグ数の多い順に上位 `limit`（未指定なら
///    [`DEFAULT_RELATED_WORKS_LIMIT`]）件を算出するフォールバックを使う
///
/// # エラー
/// クエリ実行時の DB エラーはそのまま呼び出し側に伝播する。
pub fn related_works(
    conn: &mut SqliteConnection,
    work_id: i32,
    limit: Option<usize>,
) -> QueryResult<Vec<RelatedWork>> {
    let limit = limit.unwrap_or(DEFAULT_RELATED_WORKS_LIMIT) as i64;

    let explicit = explicit_related_works(conn, work_id)?;
    if !explicit.is_empty() {
        // 各行ごとに shared_tag_count を個別クエリすると N+1 になるため、
        // explicit 全体の related_id に対する共有タグ数を1回のクエリで一括計算する。
        let related_ids: Vec<i32> = explicit.iter().map(|(id, _, _, _)| *id).collect();
        let counts = shared_tag_counts_between_many(conn, work_id, &related_ids)?;
        return Ok(explicit
            .into_iter()
            .map(|(related_id, slug, title, thumbnail)| RelatedWork {
                slug,
                title,
                thumbnail,
                shared_tag_count: counts.get(&related_id).copied().unwrap_or(0),
            })
            .collect());
    }

    shared_tag_fallback(conn, work_id, limit).map(|rows| {
        rows.into_iter()
            .map(
                |(_, slug, title, thumbnail, shared_tag_count)| RelatedWork {
                    slug,
                    title,
                    thumbnail,
                    shared_tag_count,
                },
            )
            .collect()
    })
}
