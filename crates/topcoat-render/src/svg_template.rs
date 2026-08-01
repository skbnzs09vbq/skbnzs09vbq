//! OG 画像 (`og.png` の元になる) 1200x630 の SVG を組み立てる純粋関数。
//!
//! 外部の XML シリアライズ crate には依存せず、標準ライブラリのみで文字列を組み立てる
//! (`topcoat` 側の `xml_writer` と同種のエスケープ処理をこの crate 内に小さく持つ。
//! `topcoat-render` を `topcoat` に依存させたくないための意図的な重複)。
//!
//! 抽象パターン (円) は `slug` から決定的に導出した疑似乱数シードで生成する。
//! 同じ `slug` であれば常に同じ SVG 文字列になり、ビルドの再現性を担保する。

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 630;

/// 背景色 (ダークテーマ)。
const BACKGROUND: &str = "#0b0c10";
/// 抽象パターン (円) の色候補。`slug` 由来のシードで選択する。
const ACCENT_COLORS: [&str; 4] = ["#5eead4", "#818cf8", "#f472b6", "#fbbf24"];
/// タイトル文字色。
const TITLE_COLOR: &str = "#f5f5f5";

/// XML のテキストノードとして安全な文字列にエスケープする。
///
/// `&`, `<`, `>`, `"`, `'` をすべてエスケープする (テキストノード・属性値のどちらでも
/// 安全に使えるよう、属性値相当まで含めてエスケープする)。
fn escape_xml(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// `slug` の各バイトから決定的な 64bit シードを導出する (FNV-1a 風のハッシュ)。
///
/// 暗号強度は不要で、同じ入力から常に同じ値が得られる決定性のみが要件のため、
/// 依存追加を避け自前で実装する。
fn seed_from_slug(slug: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in slug.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// 線形合同法による軽量な決定的疑似乱数生成器。
///
/// `rand` crate 相当の品質は不要 (見た目上のパターン生成にのみ使う) なため、
/// 依存を増やさず自前で実装する。
struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        // 状態が 0 のままだと以後ずっと 0 を出し続けるため、非0値に補正する。
        Self { state: seed | 1 }
    }

    /// `[0, 1)` の範囲の疑似乱数値を返す。
    fn next_f64(&mut self) -> f64 {
        // Numerical Recipes 由来の定数を用いた LCG。
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        // 上位ビットほど周期性が薄いため、上位32bitを使う。
        ((self.state >> 32) as f64) / (u32::MAX as f64 + 1.0)
    }

    fn next_range(&mut self, min: f64, max: f64) -> f64 {
        min + self.next_f64() * (max - min)
    }
}

const CIRCLE_COUNT: usize = 6;

/// `slug` から決定的に導出した抽象パターン (半透明の円) を `<circle>` 要素の並びとして生成する。
fn build_pattern_circles(slug: &str) -> String {
    let mut rng = DeterministicRng::new(seed_from_slug(slug));
    let mut out = String::new();

    for i in 0..CIRCLE_COUNT {
        let cx = rng.next_range(0.0, WIDTH as f64);
        let cy = rng.next_range(0.0, HEIGHT as f64);
        let r = rng.next_range(60.0, 260.0);
        let opacity = rng.next_range(0.08, 0.22);
        let color = ACCENT_COLORS[i % ACCENT_COLORS.len()];

        out.push_str(&format!(
            "<circle cx=\"{cx:.1}\" cy=\"{cy:.1}\" r=\"{r:.1}\" fill=\"{color}\" fill-opacity=\"{opacity:.3}\"/>\n"
        ));
    }

    out
}

/// タイトルと `slug` から 1200x630 のダークテーマ OG 画像用 SVG 文字列を組み立てる。
///
/// - タイトルは XML エスケープした上で埋め込む
/// - 抽象パターン (円) は `slug` から決定的に導出したシードで生成するため、
///   同じ `(title, slug)` の入力であれば常に同じ SVG 文字列を返す (ビルド再現性)
pub fn build_og_svg(title: &str, slug: &str) -> String {
    let escaped_title = escape_xml(title);
    let circles = build_pattern_circles(slug);

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}">
<rect x="0" y="0" width="{WIDTH}" height="{HEIGHT}" fill="{BACKGROUND}"/>
{circles}<text x="80" y="345" font-family="Archivo Black" font-size="64" fill="{TITLE_COLOR}">{escaped_title}</text>
</svg>
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_og_svg_includes_xml_escaped_title() {
        let svg = build_og_svg("A & B <script>", "sample-slug");
        assert!(svg.contains("A &amp; B &lt;script&gt;"));
        assert!(!svg.contains("A & B <script>"));
    }

    #[test]
    fn build_og_svg_is_deterministic_for_same_input() {
        let a = build_og_svg("Same Title", "same-slug");
        let b = build_og_svg("Same Title", "same-slug");
        assert_eq!(a, b);
    }

    #[test]
    fn build_og_svg_differs_for_different_slugs() {
        let a = build_og_svg("Same Title", "slug-a");
        let b = build_og_svg("Same Title", "slug-b");
        assert_ne!(a, b);
    }

    #[test]
    fn build_og_svg_has_expected_dimensions() {
        let svg = build_og_svg("Title", "slug");
        assert!(svg.contains("width=\"1200\""));
        assert!(svg.contains("height=\"630\""));
    }
}
