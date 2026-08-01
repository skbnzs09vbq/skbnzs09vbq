//! Work/Tag/Series の暫定モデル定義。
//!
//! 本来これらは #6 (Work エンティティの Diesel モデル)・#9 (タグ別一覧)・
//! #10 (シリーズ別一覧) で定義される想定だが、issue #15 着手時点でいずれも
//! 未マージのため、本 issue のスコープ内で完結させるための最小限のプレーンな
//! struct として定義している。後続 issue のマージ後は、フィールド名・型の
//! 互換性を保ったまま Diesel モデル (またはそこから変換した DTO) に差し替える。
//!
//! 追記 (issue #9 時点): タグ別一覧ページ生成のため、`Work` に `tags` (付与された
//! タグの slug 一覧) / `thumbnail` を追加している。本来 Work⇔Tag の多対多関係は
//! #2 (Tag エンティティ) の中間テーブル経由で取得する想定だが、#2・#6 いずれも
//! 未マージのため、暫定的に `Work` 自身がフラットに保持する形にしている。
//! #2/#6 マージ後は、Diesel の JOIN 結果からこれらのフィールドを構築する処理に
//! 差し替える。

/// 日時は DB からの取得値をそのまま RFC3339 (UTC, 例: `2026-01-01T00:00:00Z`) 文字列として
/// 保持する想定。chrono 等の日時 crate への依存を増やさないための暫定措置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Work {
    pub slug: String,
    pub title: String,
    pub description: String,
    /// RFC3339 (UTC) 形式
    pub created_at: String,
    /// RFC3339 (UTC) 形式。未設定 (初期シード投入直後等) の場合は
    /// フィード/サイトマップ生成時に `created_at` へフォールバックする。
    pub updated_at: Option<String>,
    /// この Work に付与されているタグの slug 一覧。
    pub tags: Vec<String>,
    /// サムネイル画像の URL・パス (未設定の場合は `None`)。
    pub thumbnail: Option<String>,
}

impl Work {
    /// `updated_at` があればそれを、なければ `created_at` を返す。
    pub fn effective_updated_at(&self) -> &str {
        effective_updated_at(self.updated_at.as_deref(), &self.created_at)
    }
}

/// `updated_at` があればそれを、なければ `created_at` を返す。
///
/// [`Work`] / `build::feed::FeedWork` のいずれも「`updated_at` が無ければ `created_at` に
/// フォールバックする」という同一のルールを持つため、両者から共通で呼び出せる純粋関数として
/// 切り出している。
pub fn effective_updated_at<'a>(updated_at: Option<&'a str>, created_at: &'a str) -> &'a str {
    updated_at.unwrap_or(created_at)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Series {
    pub slug: String,
    pub name: String,
}
