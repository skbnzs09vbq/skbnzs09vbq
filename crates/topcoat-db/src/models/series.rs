//! Series エンティティ（作品コレクション）の Diesel モデル。

use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::schema::series;

/// `series` テーブルの1レコードに対応するモデル。
#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = series)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Series {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// `series` テーブルへの新規挿入用モデル。
///
/// `created_at` / `updated_at` は SQL 側の `DEFAULT CURRENT_TIMESTAMP` に委ねるため含めない。
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = series)]
pub struct NewSeries<'a> {
    pub name: &'a str,
    pub slug: &'a str,
    pub description: &'a str,
}

impl Series {
    /// `slug` に一致する Series を1件取得する。存在しない場合は `None`。
    pub fn find_by_slug(
        conn: &mut SqliteConnection,
        target_slug: &str,
    ) -> QueryResult<Option<Series>> {
        series::table
            .filter(series::slug.eq(target_slug))
            .select(Series::as_select())
            .first(conn)
            .optional()
    }

    /// 全 Series を `name` 昇順で取得する。
    pub fn all_ordered_by_name(conn: &mut SqliteConnection) -> QueryResult<Vec<Series>> {
        series::table
            .order(series::name.asc())
            .select(Series::as_select())
            .load(conn)
    }
}
