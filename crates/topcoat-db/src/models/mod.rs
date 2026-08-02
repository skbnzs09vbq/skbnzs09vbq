//! Diesel エンティティ定義。
//!
//! テーブルごとに1モジュールを対応させる。

mod tag;
mod version;
pub mod work;

pub use tag::{NewTag, Tag};
pub use version::{NewVersion, Version};
