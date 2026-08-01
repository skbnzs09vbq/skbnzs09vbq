//! OG 画像 (`dist/works/<slug>/og.png`) の生成。
//!
//! [`crate::svg_template`] (SVG 組み立て) と [`crate::raster`] (PNG ラスタライズ) を
//! 合成する層。ファイル書き出しのような I/O を伴う関数 ([`write_og_images`]) と、
//! バイト列を返すだけの純粋寄りの関数 ([`generate_og_image`]) を分離している。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::raster::{rasterize_svg, RenderError};
use crate::svg_template::build_og_svg;

/// OG 画像 1件分の生成に必要な情報。
///
/// [`crate::models`] のようなドメインモデルは持たず、`topcoat` 側の
/// `crate::models::Work` からのマッピングは呼び出し側 (`topcoat` の `build::mod`) が行う
/// (`build::feed::FeedWork` と同じパターン)。
#[derive(Debug, Clone)]
pub struct OgWork {
    pub slug: String,
    pub title: String,
}

/// `work` から OG 画像の PNG バイト列を生成する。
///
/// SVG 組み立て (純粋関数) + PNG ラスタライズの合成のみを行い、ファイル I/O は行わない。
pub fn generate_og_image(work: &OgWork) -> Result<Vec<u8>, RenderError> {
    let svg = build_og_svg(&work.title, &work.slug);
    rasterize_svg(&svg)
}

/// `works` それぞれの OG 画像を生成し、`dist_dir/works/<slug>/og.png` として書き出す。
///
/// 書き出したファイルパスの一覧を、`works` と同じ順序で返す。
pub fn write_og_images(dist_dir: &Path, works: &[OgWork]) -> io::Result<Vec<PathBuf>> {
    let mut written_paths = Vec::with_capacity(works.len());

    for work in works {
        let png = generate_og_image(work).map_err(io::Error::other)?;

        let work_dir = dist_dir.join("works").join(&work.slug);
        fs::create_dir_all(&work_dir)?;

        let og_path = work_dir.join("og.png");
        fs::write(&og_path, png)?;

        written_paths.push(og_path);
    }

    Ok(written_paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_og_image_produces_png_bytes() {
        let work = OgWork {
            slug: "sample-work".to_string(),
            title: "Sample Work".to_string(),
        };

        let png = generate_og_image(&work).expect("generation should succeed");
        assert!(!png.is_empty());
        assert!(png.starts_with(&[0x89, 0x50, 0x4E, 0x47]));
    }

    #[test]
    fn write_og_images_writes_one_file_per_work() {
        let dir = std::env::temp_dir().join(format!(
            "topcoat-render-test-{}-{}",
            std::process::id(),
            "write_og_images_writes_one_file_per_work"
        ));
        let _ = fs::remove_dir_all(&dir);

        let works = vec![
            OgWork {
                slug: "work-a".to_string(),
                title: "Work A".to_string(),
            },
            OgWork {
                slug: "work-b".to_string(),
                title: "Work B".to_string(),
            },
        ];

        let paths = write_og_images(&dir, &works).expect("write should succeed");

        assert_eq!(paths.len(), 2);
        for path in &paths {
            assert!(path.exists());
            assert!(path.ends_with("og.png"));
        }
        assert!(dir.join("works").join("work-a").join("og.png").exists());
        assert!(dir.join("works").join("work-b").join("og.png").exists());

        let _ = fs::remove_dir_all(&dir);
    }
}
