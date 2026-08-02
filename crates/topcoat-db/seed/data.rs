//! シード投入対象データの定義。
//!
//! 本 issue（#8: 初期シードデータ投入の仕組み実装）のスコープは「仕組み」の整備であり、
//! 作品タイトル・説明文・タグ名・シリーズ構成などコンテンツの具体的な値の作り込みは
//! 対象外。以下は「ダークでおしゃれな生成アート」というコンセプトに沿った、SSG
//! ビルドパイプラインの検証に足る最小限のプレースホルダデータであり、後続の
//! 「コンテンツ生成」issue で本格的な値に差し替える前提であることに注意すること。
//!
//! `series` テーブル（issue #1, PR #28）は本 issue 着手時点で `origin/main` 未マージ
//! のため、Series の投入は行わず、全 Work の `series_id` を `None` にしている。
//! #28 がマージされ次第、Series 投入を追加する追従対応（`crates/topcoat_db::models::series`
//! の `Series`/`NewSeries` を使った投入関数の追加、および各 Work への `series_id` 割り当て）
//! が必要になる。

use chrono::{Duration, NaiveDate, NaiveDateTime};

use topcoat_db::models::work::{generate_slug, Work};

/// シード投入するタグの `(id, name)`。`slug` は insert 時に [`generate_slug`] から導出する。
pub const TAG_SEEDS: &[(i32, &str)] = &[
    (1, "ジェネラティブアート"),
    (2, "ダーク"),
    (3, "モノクローム"),
    (4, "グリッチ"),
    (5, "ミニマル"),
    (6, "幾何学"),
    (7, "ノイズ"),
    (8, "アンビエント"),
];

/// シード投入する Work の `(id, title)`。20〜30件目安（現状24件）。
/// `title` はプレースホルダであり、内容の作り込みは本 issue のスコープ外。
const WORK_SEEDS: &[(i32, &str)] = &[
    (1, "Obsidian Bloom"),
    (2, "Voidframe Drift"),
    (3, "Null Horizon"),
    (4, "Chroma Static"),
    (5, "Ashen Lattice"),
    (6, "Ember Fractal"),
    (7, "Monochrome Pulse"),
    (8, "Silent Grid"),
    (9, "Nocturne Weave"),
    (10, "Fractured Halo"),
    (11, "Ink Drift"),
    (12, "Glass Noise"),
    (13, "Umbra Field"),
    (14, "Static Bloom"),
    (15, "Hollow Spectrum"),
    (16, "Carbon Drift"),
    (17, "Vantablack Study"),
    (18, "Fragment Zero"),
    (19, "Midnight Circuit"),
    (20, "Charcoal Echo"),
    (21, "Onyx Cascade"),
    (22, "Wraith Signal"),
    (23, "Slate Fracture"),
    (24, "Eclipse Weave"),
];

/// シードデータ全体の基準となる固定日時。
///
/// `chrono::Utc::now()` ではなく固定値を使うことで、複数回実行しても常に同一の
/// `created_at`/`updated_at` になり、冪等性テスト（`tests/seed.rs`）で内容の完全一致を
/// 検証できるようにしている。
fn base_datetime() -> NaiveDateTime {
    NaiveDate::from_ymd_opt(2026, 1, 1)
        .expect("有効な日付です")
        .and_hms_opt(0, 0, 0)
        .expect("有効な時刻です")
}

/// Work の `id` から決定的な日時を導出する（`id` が大きいほど後の時刻になる）。
fn work_timestamp(id: i32) -> NaiveDateTime {
    base_datetime() + Duration::minutes(i64::from(id))
}

/// 投入対象の [`Work`] 一覧を構築する。
///
/// `params` は後続のレンダリング側が参照できる最小限の JSON 文字列
/// （`seed`/`palette` のみ）とし、値そのものの作り込みは行わない。
pub fn works() -> Vec<Work> {
    WORK_SEEDS
        .iter()
        .map(|&(id, title)| {
            let timestamp = work_timestamp(id);
            Work {
                id,
                title: title.to_string(),
                slug: generate_slug(title),
                description: Some(format!(
                    "{title} — ダークトーンを基調とした生成アート作品のプレースホルダ説明文。"
                )),
                series_id: None,
                created_at: timestamp,
                updated_at: timestamp,
                thumbnail: Some(format!("thumb-{id:02}.png")),
                params: Some(format!(r#"{{"seed":{id},"palette":"dark"}}"#)),
            }
        })
        .collect()
}

/// `work_tags` へ投入する `(work_id, tag_id)` の組。
///
/// 各 Work に1〜2件のタグを、`id` から決定的に導出して割り当てる
/// （同一 Work に同じタグが重複しないよう、算出結果が一致する場合は1件のみにする）。
pub fn work_tag_pairs() -> Vec<(i32, i32)> {
    let tag_count = TAG_SEEDS.len() as i32;

    WORK_SEEDS
        .iter()
        .flat_map(|&(work_id, _)| {
            let primary = ((work_id - 1) % tag_count) + 1;
            let secondary = ((work_id + 2) % tag_count) + 1;

            if primary == secondary {
                vec![(work_id, primary)]
            } else {
                vec![(work_id, primary), (work_id, secondary)]
            }
        })
        .collect()
}

/// `versions` へ投入する `(work_id, version_label, changelog)` の組。
///
/// 全 Work に初回リリース（`v1.0`）を1件、うち最初の6件には変遷履歴の検証用に
/// マイナーアップデート（`v1.1`）をもう1件追加する。
pub fn version_seeds() -> Vec<(i32, &'static str, &'static str)> {
    let mut versions: Vec<(i32, &'static str, &'static str)> = WORK_SEEDS
        .iter()
        .map(|&(work_id, _)| (work_id, "v1.0", "初回リリース。"))
        .collect();

    versions.extend(WORK_SEEDS.iter().take(6).map(|&(work_id, _)| {
        (
            work_id,
            "v1.1",
            "パラメータ調整によるマイナーアップデート。",
        )
    }));

    versions
}
