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
///
/// `tags` / `series` / `params` / `related_works` は issue #12 (作品詳細静的ページ生成)
/// 着手時点で #6 (Work エンティティの Diesel モデル)・#7 (関連作品算出ロジック) が
/// いずれも未マージのため、実データと接続されていない暫定フィールドである。
/// 後続 issue のマージ後は、フィールド名・型の互換性を保ったまま実データ取得処理に
/// 差し替える想定。
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
    /// この Work に紐づくタグ一覧。#9 (タグ別一覧) マージ後に実データへ差し替える想定の暫定フィールド。
    /// 全文検索インデックス (`search-index.json`) 生成時には表示名 (`name`) のみ抽出して使う。
    /// タグ別一覧ページ生成時には各 [`Tag::slug`] でフィルタする。
    pub tags: Vec<Tag>,
    /// サムネイル画像の URL・パス (未設定の場合は `None`)。タグ別一覧ページで使用する。
    pub thumbnail: Option<String>,
    /// この Work が属するシリーズ。存在しない場合は `None`。
    /// #10 (シリーズ別一覧) マージ後に実データへ差し替える想定の暫定フィールド。
    /// 全文検索インデックス (`search-index.json`) 生成時には表示名 (`name`) のみ抽出して使う。
    pub series: Option<Series>,
    /// 所属する [`Series`] の `slug` (FK)。シリーズ別一覧ページの絞り込みに使う。
    /// 未所属の場合は `None`。
    pub series_slug: Option<String>,
    /// 作品の生成パラメータ。#6 側でスキーマが未確定のため、本 issue では
    /// `serde_json::Value` による自由形式の暫定表現とする。
    pub params: serde_json::Value,
    /// 関連作品一覧。#7 (関連作品算出ロジック) マージ後に実データへ差し替える想定の暫定フィールド。
    pub related_works: Vec<RelatedWorkRef>,
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

/// Work に紐づくバージョン (更新履歴) 1件分の情報。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    /// 対象の Work の ID (DB 上の主キー)。
    pub work_id: i64,
    /// バージョン表記 (例: "v1.0")。
    pub version: String,
    pub note: String,
    /// RFC3339 (UTC) 形式
    pub created_at: String,
}

/// 関連作品への参照。一覧表示に必要な最小限の情報のみを持つ。
///
/// #7 (関連作品算出ロジック) マージ後に実データへ差し替える想定の暫定 struct。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelatedWorkRef {
    pub slug: String,
    pub title: String,
}
