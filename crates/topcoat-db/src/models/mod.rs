//! Diesel エンティティ定義。
//!
//! テーブルごとに1モジュールを対応させる。

mod version;
pub mod work;

pub use version::{NewVersion, Version};
