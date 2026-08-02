//! 既存データの削除（DELETE）処理。
//!
//! シード投入を冪等にするため、投入前に既存データを FK 依存順（子→親）で全削除する。
//! `series` テーブル（issue #1, PR #28）は本 issue 着手時点で未マージのため対象外。
//! マージ後は `work_tags` → `related_works` → `versions` → `works` → `tags` → `series`
//! の順に拡張すること。

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use topcoat_db::schema::{related_works, tags, versions, work_tags, works};

/// 既存のシードデータを FK 依存順（子→親）ですべて削除する。
///
/// `establish_connection_at` が接続確立時に `PRAGMA foreign_keys = ON;` を発行しているため、
/// 削除順序を誤ると FOREIGN KEY 制約違反で失敗する。
pub fn reset(conn: &mut SqliteConnection) -> QueryResult<()> {
    diesel::delete(work_tags::table).execute(conn)?;
    diesel::delete(related_works::table).execute(conn)?;
    diesel::delete(versions::table).execute(conn)?;
    diesel::delete(works::table).execute(conn)?;
    diesel::delete(tags::table).execute(conn)?;
    Ok(())
}
