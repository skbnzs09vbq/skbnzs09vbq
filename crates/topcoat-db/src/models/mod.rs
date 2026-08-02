//! Diesel エンティティ定義。
//!
//! テーブルごとに1モジュールを対応させる。

mod series;
mod tag;
mod version;
pub mod work;

pub use series::{NewSeries, Series};
pub use tag::{NewTag, Tag};
pub use version::{NewVersion, Version};
