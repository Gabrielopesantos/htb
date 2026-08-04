//! Strips the promotional noise YouTube titles carry, such as `(Official Video)`,
//! `[MV]`, `| Official Music Video` - so it does not end up in the catalog,
//! the ID3 title, or the filename.

/// Whether a bracketed group's contents are promotional noise rather than part
/// of the title.
fn is_noise(inner: &str) -> bool {
    let inner = inner.trim().to_lowercase();
    let inner = inner.split_whitespace().collect::<Vec<_>>().join(" ");

    if inner.is_empty() {
        return false;
    }

    // "official video", "official music video", "official audio",
    // "official lyric video", "official visualizer", ...
    if inner.contains("official") {
        return true;
    }

    if inner.contains("lyric") || inner.contains("music video") {
        return true;
    }

    matches!(
        inner.as_str(),
        "mv" | "m/v"
            | "audio"
            | "video"
            | "visualizer"
            | "visualiser"
            | "hd"
            | "hq"
            | "4k"
            | "8k"
            | "1080p"
            | "720p"
    )
}

/// Removes noise groups and tidies the leftover whitespace and separators.
/// Returns the title unchanged when there is nothing to strip.
pub fn clean(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    let mut rest = title;

    while let Some(open) = rest.find(['(', '[']) {
        let close_char = if rest.as_bytes()[open] == b'(' {
            ')'
        } else {
            ']'
        };

        let Some(close) = rest[open..].find(close_char).map(|i| open + i) else {
            // Unbalanced bracket, nothing more to scan.
            break;
        };

        out.push_str(&rest[..open]);
        if !is_noise(&rest[open + 1..close]) {
            out.push_str(&rest[open..=close]);
        }
        rest = &rest[close + 1..];
    }
    out.push_str(rest);

    // Trailing "| Official Video" style segments carry no brackets.
    if let Some(bar) = out.rfind('|') {
        if is_noise(&out[bar + 1..]) {
            out.truncate(bar);
        }
    }

    tidy(&out)
}

/// Collapses the whitespace and dangling separators left behind by removals.
fn tidy(s: &str) -> String {
    let collapsed = s.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed
        .trim_matches(|c: char| c.is_whitespace() || c == '-' || c == '|')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_common_noise() {
        let cases = [
            ("Example Song (Official Video)", "Example Song"),
            ("Example Song [Official Music Video]", "Example Song"),
            (
                "Example Artist - 'Some Song' [MV]",
                "Example Artist - 'Some Song'",
            ),
            ("Some Song (Official Audio)", "Some Song"),
            ("Some Song (Lyric Video)", "Some Song"),
            ("Some Song (HD)", "Some Song"),
            ("Some Song | Official Video", "Some Song"),
        ];

        for (input, expected) in cases {
            assert_eq!(clean(input), expected, "input: {input}");
        }
    }

    #[test]
    fn preserves_the_video_id_group() {
        assert_eq!(
            clean("Example Song (Official Video) [aBcD1234xyz]"),
            "Example Song [aBcD1234xyz]"
        );
    }

    #[test]
    fn preserves_meaningful_parentheticals() {
        for title in [
            "Song (Live)",
            "Song (Acoustic)",
            "Song (Remastered 2011)",
            "Song (feat. Someone)",
            "Song (Radio Edit)",
        ] {
            assert_eq!(clean(title), title);
        }
    }

    /// Known limitation: only bracketed groups and trailing `|` segments are
    /// considered, so bare suffixes survive. Stripping those unquoted would
    /// risk eating real title text.
    #[test]
    fn leaves_unbracketed_noise_alone() {
        assert_eq!(
            clean("Example Artist - 'Some Song' M/V"),
            "Example Artist - 'Some Song' M/V"
        );
    }

    #[test]
    fn leaves_clean_titles_untouched() {
        assert_eq!(clean("Example Song"), "Example Song");
    }

    #[test]
    fn handles_unbalanced_and_empty_groups() {
        assert_eq!(clean("Song (unclosed"), "Song (unclosed");
        assert_eq!(clean("Song ()"), "Song ()");
        assert_eq!(clean(""), "");
    }
}
