//! OG 画像生成 (SVG 組み立て + PNG ラスタライズ) の統合テスト。
//!
//! crate 公開 API (`topcoat_render`) を通して検証する。

use topcoat_render::{build_og_svg, rasterize_svg};

const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

#[test]
fn build_og_svg_contains_xml_escaped_title() {
    let svg = build_og_svg("Rust & <SVG> \"quotes\"", "escape-test");

    assert!(svg.contains("Rust &amp; &lt;SVG&gt; &quot;quotes&quot;"));
    assert!(!svg.contains("Rust & <SVG>"));
}

#[test]
fn rasterize_svg_output_starts_with_png_signature_and_is_not_empty() {
    let svg = build_og_svg("Integration Test Title", "integration-test-slug");

    let png = rasterize_svg(&svg).expect("rasterize_svg should succeed for a valid SVG");

    assert!(!png.is_empty());
    assert!(png.starts_with(&PNG_SIGNATURE));
}
