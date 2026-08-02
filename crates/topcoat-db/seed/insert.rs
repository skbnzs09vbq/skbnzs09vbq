//! 投入（INSERT）処理。
//!
//! Tags → Works → work_tags → Versions の順で投入する
//! （`series` は issue #1, PR #28 が本 issue 着手時点で未マージのため対象外。
//! マージ後は Series → Tags → Works → work_tags → Versions の順に拡張すること）。
//!
//! `tags` には現時点で `NewTag` のような Insertable 専用モデルが無いため、
//! `tests/related.rs` の生スキーマ直接 insert パターンを踏襲し、`schema::tags` を
//! 直接使う。将来 `NewTag` が追加された場合はそちらへの置き換えを検討すること。

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use topcoat_db::models::work::generate_slug;
use topcoat_db::models::NewVersion;
use topcoat_db::schema::{tags, versions, work_tags, works};

use crate::data;

/// [`data::TAG_SEEDS`] を投入する。
pub fn insert_tags(conn: &mut SqliteConnection) -> QueryResult<usize> {
    let mut inserted = 0;
    for &(id, name) in data::TAG_SEEDS {
        inserted += diesel::insert_into(tags::table)
            .values((
                tags::id.eq(id),
                tags::name.eq(name),
                tags::slug.eq(generate_slug(name)),
            ))
            .execute(conn)?;
    }
    Ok(inserted)
}

/// [`data::works`] を投入する。
pub fn insert_works(conn: &mut SqliteConnection) -> QueryResult<usize> {
    let mut inserted = 0;
    for work in data::works() {
        inserted += diesel::insert_into(works::table)
            .values(&work)
            .execute(conn)?;
    }
    Ok(inserted)
}

/// [`data::work_tag_pairs`] を投入する。
pub fn insert_work_tags(conn: &mut SqliteConnection) -> QueryResult<usize> {
    let mut inserted = 0;
    for (work_id, tag_id) in data::work_tag_pairs() {
        inserted += diesel::insert_into(work_tags::table)
            .values((work_tags::work_id.eq(work_id), work_tags::tag_id.eq(tag_id)))
            .execute(conn)?;
    }
    Ok(inserted)
}

/// [`data::version_seeds`] を投入する。
pub fn insert_versions(conn: &mut SqliteConnection) -> QueryResult<usize> {
    let mut inserted = 0;
    for (work_id, version_label, changelog) in data::version_seeds() {
        inserted += diesel::insert_into(versions::table)
            .values(&NewVersion {
                work_id,
                version_label,
                changelog,
            })
            .execute(conn)?;
    }
    Ok(inserted)
}

/// Tags → Works → work_tags → Versions の順で全データを投入する。
pub fn insert_all(conn: &mut SqliteConnection) -> QueryResult<()> {
    insert_tags(conn)?;
    insert_works(conn)?;
    insert_work_tags(conn)?;
    insert_versions(conn)?;
    Ok(())
}
