//! `works` テーブルに対応する Diesel エンティティ。

use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::schema::works;

/// `works` テーブルの1レコードを表すエンティティ。
///
/// `Queryable`/`Selectable`/`Insertable` をすべて同一構造体に付与しているため、
/// insert 時は `id` も明示的に指定する必要がある（`schema::works::id` は
/// autoincrement を前提とした `Nullable<Integer>` ではなく `Integer` として
/// 手動補正済み。詳細は `schema.rs` のコメントを参照）。
#[derive(Queryable, Selectable, Insertable, Debug, Clone, PartialEq, Eq)]
#[diesel(table_name = works)]
pub struct Work {
    pub id: i32,
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub series_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub thumbnail: Option<String>,
    pub params: Option<String>,
}

/// タイトルから URL スラッグを生成する。
///
/// [`slug::slugify`] のラッパー。スラッグ生成方法をこの関数にカプセル化することで、
/// 呼び出し側（シードデータ投入処理など）が `slug` crate に直接依存しなくて済むようにする。
pub fn generate_slug(title: &str) -> String {
    slug::slugify(title)
}

#[cfg(test)]
mod tests {
    use super::generate_slug;

    #[test]
    fn generate_slug_converts_title_to_kebab_case() {
        assert_eq!(generate_slug("Hello World"), "hello-world");
    }

    #[test]
    fn generate_slug_handles_symbols_and_whitespace() {
        // 記号・空白はハイフンに正規化され、連続するハイフンは1つにまとめられる。
        assert_eq!(
            generate_slug("Hello, World!!  Part 1"),
            "hello-world-part-1"
        );
    }

    #[test]
    fn generate_slug_transliterates_non_latin_characters() {
        // 日本語等のラテン文字ではない文字は、slug crate（deunicode 経由）により
        // 読みに基づいたラテン文字表記へ変換される（除去はされない）。
        let result = generate_slug("作品タイトル");
        assert!(!result.is_empty());
        assert!(result.is_ascii());
    }
}
