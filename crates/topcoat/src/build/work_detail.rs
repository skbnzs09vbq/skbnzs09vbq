//! 作品詳細静的ページ (`dist/works/<slug>/index.html`) の生成。
//!
//! [`crate::models::Work`] から [`topcoat_render::work_detail::WorkDetailContext`] を
//! 組み立て、`topcoat-render` 側のテンプレートレンダリングを呼び出して書き出す。

use std::fs;
use std::io;
use std::path::Path;

use tera::Tera;
use topcoat_render::work_detail::{
    render_work_detail, RelatedWorkRef as RenderRelatedWorkRef, SeriesRef as RenderSeriesRef,
    TagRef as RenderTagRef, WorkDetailContext,
};

use crate::models::Work;

/// `works` それぞれについて `dist_dir/works/<slug>/index.html` を書き出す。
///
/// 出力先ディレクトリ (`dist_dir/works/<slug>/`) が存在しなければ作成する。
pub fn write_work_detail_pages(dist_dir: &Path, tera: &Tera, works: &[Work]) -> io::Result<()> {
    for work in works {
        let ctx = WorkDetailContext {
            slug: work.slug.clone(),
            title: work.title.clone(),
            description: work.description.clone(),
            tags: work
                .tags
                .iter()
                .map(|tag| RenderTagRef {
                    slug: tag.slug.clone(),
                    name: tag.name.clone(),
                })
                .collect(),
            series: work.series.as_ref().map(|series| RenderSeriesRef {
                slug: series.slug.clone(),
                name: series.name.clone(),
            }),
            params: work.params.clone(),
            related_works: work
                .related_works
                .iter()
                .map(|related| RenderRelatedWorkRef {
                    slug: related.slug.clone(),
                    title: related.title.clone(),
                })
                .collect(),
        };

        let html = render_work_detail(tera, &ctx).map_err(io::Error::other)?;

        let work_dir = dist_dir.join("works").join(&work.slug);
        fs::create_dir_all(&work_dir)?;
        fs::write(work_dir.join("index.html"), html)?;
    }

    Ok(())
}
