//! `assets/` (静的アセット: CSS/JS 等) を `dist_dir/assets/` へコピーする処理。
//!
//! issue #30 (フロントエンド基盤) により、`topcoat-render` の `templates/layout.html` が
//! 参照する `/assets/css/tokens.css` 等の静的アセットを配信できるよう、ワークスペースルート
//! の `assets/` ディレクトリを再帰的に `dist_dir/assets/` へコピーする。
//!
//! コピー元は `CARGO_MANIFEST_DIR` (このクレートのルート = `crates/topcoat`) を基準に
//! ワークスペースルートを解決するため、`topcoat build` の実行時カレントディレクトリに
//! 依存しない。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// ワークスペースルートの `assets/` ディレクトリの絶対パス。
///
/// `CARGO_MANIFEST_DIR` (`crates/topcoat`) から2階層上がワークスペースルート。
fn source_assets_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
}

/// ワークスペースルートの `assets/` を `dist_dir/assets/` へ再帰的にコピーする。
///
/// コピー元の `assets/` ディレクトリが存在しない場合は何もしない (no-op)。
pub fn write_assets(dist_dir: &Path) -> io::Result<()> {
    write_assets_from(&source_assets_dir(), dist_dir)
}

/// `src` を `dist_dir/assets/` へ再帰的にコピーする ([`write_assets`] の本体、テスト用に
/// コピー元ディレクトリを差し替え可能にしている)。
///
/// `src` が存在しない場合は何もしない (no-op)。
fn write_assets_from(src: &Path, dist_dir: &Path) -> io::Result<()> {
    if !src.is_dir() {
        return Ok(());
    }

    let dest = dist_dir.join("assets");
    copy_dir_recursive(src, &dest)
}

/// `src` 配下のファイル・ディレクトリを再帰的に `dest` へコピーする。
fn copy_dir_recursive(src: &Path, dest: &Path) -> io::Result<()> {
    fs::create_dir_all(dest)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dest.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), &dest_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_assets_from_copies_nested_files_under_dist_assets() {
        let src_dir = tempfile::tempdir().expect("src tempdir");
        let dist_dir = tempfile::tempdir().expect("dist tempdir");

        fs::create_dir_all(src_dir.path().join("css")).unwrap();
        fs::write(src_dir.path().join("css/tokens.css"), "body{}").unwrap();
        fs::write(src_dir.path().join("root.txt"), "root").unwrap();

        write_assets_from(src_dir.path(), dist_dir.path()).unwrap();

        assert_eq!(
            fs::read_to_string(dist_dir.path().join("assets/css/tokens.css")).unwrap(),
            "body{}"
        );
        assert_eq!(
            fs::read_to_string(dist_dir.path().join("assets/root.txt")).unwrap(),
            "root"
        );
    }

    #[test]
    fn write_assets_from_is_noop_when_source_dir_missing() {
        let dist_dir = tempfile::tempdir().expect("dist tempdir");
        let missing_src = dist_dir.path().join("does-not-exist");

        write_assets_from(&missing_src, dist_dir.path()).unwrap();

        assert!(!dist_dir.path().join("assets").exists());
    }

    #[test]
    fn write_assets_copies_workspace_assets_dir_into_dist() {
        let dist_dir = tempfile::tempdir().expect("dist tempdir");

        write_assets(dist_dir.path()).unwrap();

        assert!(dist_dir.path().join("assets/css/tokens.css").is_file());
    }
}
