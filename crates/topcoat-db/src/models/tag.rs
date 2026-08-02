//! `Tag` エンティティ（Work との多対多リレーション）。
//!
//! `work_tags` 中間テーブルを介して [`crate::models::work::Work`] と多対多の関係を持つ
//! （Work⇔Tag の実際の取得関数は [`crate::queries::tag`] を参照）。`tags` テーブル自体は
//! issue #7（関連作品算出ロジック）が暫定実装として先行作成済みで、本 issue（#2）が
//! Diesel モデル・クエリ関数を正式実装する。

use diesel::prelude::*;

use crate::schema::tags;

/// `tags` テーブルの1行を表すモデル。
#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = tags)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

/// `tags` テーブルへの新規挿入用モデル。
///
/// `id` は SQLite の `INTEGER PRIMARY KEY`（rowid）が自動採番するため、ここには含めない。
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = tags)]
pub struct NewTag<'a> {
    pub name: &'a str,
    pub slug: &'a str,
}

impl Tag {
    /// `slug` に一致する Tag を1件取得する。存在しなければ `None`。
    pub fn find_by_slug(
        connection: &mut SqliteConnection,
        target_slug: &str,
    ) -> QueryResult<Option<Tag>> {
        use crate::schema::tags::dsl::{slug, tags};

        tags.filter(slug.eq(target_slug))
            .first::<Tag>(connection)
            .optional()
    }

    /// 全 Tag を `name` 昇順（同名時は `id` 昇順でタイブレーク）で取得する。
    pub fn all_ordered_by_name(connection: &mut SqliteConnection) -> QueryResult<Vec<Tag>> {
        use crate::schema::tags::dsl::{id, name, tags};

        tags.order((name.asc(), id.asc())).load::<Tag>(connection)
    }
}
