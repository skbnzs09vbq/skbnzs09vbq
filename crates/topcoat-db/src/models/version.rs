//! `Version` エンティティ（作品の変遷履歴・changelog）。
//!
//! 1つの Work（`work_id`）に対して複数の Version を1対多で保持する（issue #5）。
//! `works` テーブル自体は別 issue（#6）で追加されるため、`Work` への実際の
//! `#[diesel(belongs_to(Work))]` 関連付けはそちら側の実装後に行う。

use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::schema::versions;

/// `versions` テーブルの1行を表すモデル。
#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = versions)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Version {
    pub id: i32,
    pub work_id: i32,
    pub version_label: String,
    pub changelog: String,
    pub created_at: NaiveDateTime,
}

/// `versions` テーブルへの新規挿入用モデル。
///
/// `id` は SQLite の `INTEGER PRIMARY KEY`（rowid）が自動採番し、`created_at` は
/// カラムの `DEFAULT CURRENT_TIMESTAMP` に委ねるため、どちらもここには含めない。
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = versions)]
pub struct NewVersion<'a> {
    pub work_id: i32,
    pub version_label: &'a str,
    pub changelog: &'a str,
}

impl Version {
    /// 指定した `work_id` の Version 一覧を、`created_at` の時系列順（昇順）で取得する。
    ///
    /// `created_at`（秒単位精度）が同一になるケースに備え、`id`（挿入順）を副次的な
    /// ソートキーとして使い、結果の順序を決定的にする。
    pub fn for_work(
        connection: &mut SqliteConnection,
        target_work_id: i32,
    ) -> QueryResult<Vec<Version>> {
        use crate::schema::versions::dsl::{created_at, id, versions, work_id};

        versions
            .filter(work_id.eq(target_work_id))
            .order((created_at.asc(), id.asc()))
            .load::<Version>(connection)
    }
}
