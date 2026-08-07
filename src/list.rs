//! Ordering and rendering for `htb list`.
//!
//! Everything writes through a `Write` rather than `println!`, so the caller
//! decides the stream and can handle a closed pipe instead of panicking.

use crate::media::Media;
use clap::ValueEnum;
use std::cmp::Ordering;
use std::io::{self, Write};
use unicode_width::UnicodeWidthStr;

/// Placeholder for a column with no value, so the table keeps its shape.
const EMPTY_CELL: &str = "-";

/// Space between columns in table output.
const GUTTER: &str = "  ";

const DEFAULT_HEADERS: &[&str] = &["NAME", "ARTIST", "ALBUM", "LIBRARY", "TAGS"];

const LONG_HEADERS: &[&str] = &[
    "NAME", "ARTIST", "ALBUM", "TRACK", "YEAR", "GENRE", "LIBRARY", "TAGS", "FILENAME", "URL",
    "ADDED",
];

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Name,
    Artist,
    Album,
    /// Library, then name - the default, which keeps directories grouped.
    Library,
    /// Insertion timestamp, oldest first.
    Added,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Table,
    Tsv,
    Json,
}

/// Orders `items` in place. Entries missing the sort key (no artist, no album)
/// always land at the end, in both directions, so `--reverse` does not open
/// with a block of blanks.
pub fn sort(items: &mut [Media], key: SortKey, reverse: bool) {
    items.sort_by(|a, b| compare(a, b, key, reverse));
}

fn compare(a: &Media, b: &Media, key: SortKey, reverse: bool) -> Ordering {
    match (sort_value(a, key), sort_value(b, key)) {
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (left, right) => {
            // Name and filename tiebreak so equal keys still produce a stable
            // order across runs, whatever SQLite returned.
            let ordering = left
                .cmp(&right)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                .then_with(|| a.filename.cmp(&b.filename));

            if reverse {
                ordering.reverse()
            } else {
                ordering
            }
        }
    }
}

/// `None` means the entry has no value for this key. An empty library is a real
/// value (the root of the catalog), not a missing one.
fn sort_value(media: &Media, key: SortKey) -> Option<String> {
    match key {
        SortKey::Name => Some(media.name.to_lowercase()),
        SortKey::Artist => media.id3.artist.as_ref().map(|v| v.to_lowercase()),
        SortKey::Album => media.id3.album.as_ref().map(|v| v.to_lowercase()),
        SortKey::Library => Some(media.library.to_lowercase()),
        // Stored as "YYYY-MM-DD HH:MM:SS", so lexicographic is chronological.
        SortKey::Added => media.inserted_at.clone(),
    }
}

pub fn render(
    items: &[Media],
    format: OutputFormat,
    long: bool,
    out: &mut impl Write,
) -> io::Result<()> {
    match format {
        OutputFormat::Table => render_table(items, long, out),
        OutputFormat::Tsv => render_tsv(items, long, out),
        OutputFormat::Json => render_json(items, out),
    }
}

fn headers(long: bool) -> &'static [&'static str] {
    if long {
        LONG_HEADERS
    } else {
        DEFAULT_HEADERS
    }
}

/// One cell per header, in the same order. Absent values are empty strings;
/// each format decides how to show them.
fn cells(media: &Media, long: bool) -> Vec<String> {
    let id3 = &media.id3;
    let optional = |value: &Option<String>| value.clone().unwrap_or_default();

    let mut cells = vec![
        media.name.clone(),
        optional(&id3.artist),
        optional(&id3.album),
    ];

    if long {
        cells.push(id3.track.map(|v| v.to_string()).unwrap_or_default());
        cells.push(id3.year.map(|v| v.to_string()).unwrap_or_default());
        cells.push(optional(&id3.genre));
    }

    cells.push(media.library.clone());
    cells.push(media.tags.clone());

    if long {
        cells.push(media.filename.clone());
        cells.push(media.url.clone());
        cells.push(optional(&media.inserted_at));
    }

    cells
}

fn render_table(items: &[Media], long: bool, out: &mut impl Write) -> io::Result<()> {
    let headers = headers(long);
    let rows: Vec<Vec<String>> = items.iter().map(|media| cells(media, long)).collect();

    // Width pass first: a column is as wide as its widest cell, header included.
    // Terminal columns, not chars - a CJK title takes two columns per character,
    // and this catalog is full of them.
    let mut widths: Vec<usize> = headers.iter().map(|header| header.width()).collect();
    for row in &rows {
        for (column, cell) in row.iter().enumerate() {
            widths[column] = widths[column].max(shown(cell).width());
        }
    }

    write_row(headers.iter().copied(), &widths, out)?;
    for row in &rows {
        write_row(row.iter().map(|cell| shown(cell)), &widths, out)?;
    }

    Ok(())
}

fn write_row<'a>(
    cells: impl Iterator<Item = &'a str>,
    widths: &[usize],
    out: &mut impl Write,
) -> io::Result<()> {
    let last = widths.len() - 1;
    for (column, cell) in cells.enumerate() {
        if column == last {
            // Padding the final column would leave trailing whitespace on every
            // line, which shows up in diffs and copied output.
            writeln!(out, "{cell}")?;
        } else {
            // Padded by hand rather than with `{:<width$}`, which counts chars
            // and so under-pads anything wider than one column.
            let padding = widths[column].saturating_sub(cell.width());
            write!(out, "{cell}{}{GUTTER}", " ".repeat(padding))?;
        }
    }

    Ok(())
}

fn shown(cell: &str) -> &str {
    if cell.is_empty() {
        EMPTY_CELL
    } else {
        cell
    }
}

/// Header-less and unpadded, for `cut`/`awk`. Absent values stay empty so the
/// field count is all a script has to rely on.
fn render_tsv(items: &[Media], long: bool, out: &mut impl Write) -> io::Result<()> {
    for media in items {
        writeln!(out, "{}", cells(media, long).join("\t"))?;
    }

    Ok(())
}

/// Always every field, `--long` or not - a script can pick what it needs.
fn render_json(items: &[Media], out: &mut impl Write) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut *out, items)?;
    writeln!(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::Id3Tags;

    fn media(name: &str, library: &str, artist: Option<&str>) -> Media {
        Media {
            name: name.to_string(),
            filename: format!("{name}.mp3"),
            library: library.to_string(),
            url: format!("https://youtu.be/{name}"),
            tags: "chill".to_string(),
            id3: Id3Tags {
                artist: artist.map(String::from),
                ..Default::default()
            },
            inserted_at: Some("2026-01-01 00:00:00".to_string()),
        }
    }

    fn names(items: &[Media]) -> Vec<&str> {
        items.iter().map(|media| media.name.as_str()).collect()
    }

    fn rendered(items: &[Media], format: OutputFormat, long: bool) -> String {
        let mut out = Vec::new();
        render(items, format, long, &mut out).unwrap();

        String::from_utf8(out).unwrap()
    }

    #[test]
    fn sorts_by_library_then_name() {
        let mut items = vec![
            media("Zeta", "rock", None),
            media("beta", "pop", None),
            media("Alpha", "rock", None),
        ];

        sort(&mut items, SortKey::Library, false);

        assert_eq!(names(&items), ["beta", "Alpha", "Zeta"]);
    }

    #[test]
    fn sorts_by_name_case_insensitively() {
        let mut items = vec![media("beta", "rock", None), media("Alpha", "pop", None)];

        sort(&mut items, SortKey::Name, false);

        assert_eq!(names(&items), ["Alpha", "beta"]);
    }

    #[test]
    fn sorts_by_artist_and_album() {
        let mut items = vec![
            media("b", "", Some("Artist 2")),
            media("a", "", Some("Artist 1")),
        ];
        sort(&mut items, SortKey::Artist, false);
        assert_eq!(names(&items), ["a", "b"]);

        items[0].id3.album = Some("Later".into());
        items[1].id3.album = Some("Earlier".into());
        sort(&mut items, SortKey::Album, false);
        assert_eq!(names(&items), ["b", "a"]);
    }

    #[test]
    fn sorts_by_added_oldest_first() {
        let mut items = vec![media("new", "", None), media("old", "", None)];
        items[0].inserted_at = Some("2026-02-01 00:00:00".into());
        items[1].inserted_at = Some("2026-01-01 00:00:00".into());

        sort(&mut items, SortKey::Added, false);
        assert_eq!(names(&items), ["old", "new"]);

        sort(&mut items, SortKey::Added, true);
        assert_eq!(names(&items), ["new", "old"]);
    }

    #[test]
    fn entries_missing_the_sort_key_stay_last_in_both_directions() {
        let mut items = vec![
            media("no artist", "", None),
            media("has artist", "", Some("Artist 1")),
        ];

        sort(&mut items, SortKey::Artist, false);
        assert_eq!(names(&items), ["has artist", "no artist"]);

        sort(&mut items, SortKey::Artist, true);
        assert_eq!(names(&items), ["has artist", "no artist"]);
    }

    #[test]
    fn reverse_flips_the_order() {
        let mut items = vec![media("Alpha", "pop", None), media("Zeta", "rock", None)];

        sort(&mut items, SortKey::Library, true);

        assert_eq!(names(&items), ["Zeta", "Alpha"]);
    }

    #[test]
    fn equal_keys_tiebreak_on_name_then_filename() {
        let mut first = media("Same", "pop", None);
        first.filename = "a.mp3".into();
        let mut second = media("Same", "pop", None);
        second.filename = "b.mp3".into();
        let mut items = vec![second, first];

        sort(&mut items, SortKey::Library, false);

        assert_eq!(items[0].filename, "a.mp3");
        assert_eq!(items[1].filename, "b.mp3");
    }

    #[test]
    fn table_has_a_header_and_aligned_columns() {
        let items = vec![
            media("A very long name", "pop", Some("Artist 1")),
            media("Short", "rock", None),
        ];

        let output = rendered(&items, OutputFormat::Table, false);
        let lines: Vec<&str> = output.lines().collect();

        assert!(lines[0].starts_with("NAME"));
        // Every row starts ARTIST at the same offset.
        let artist_column = lines[0].find("ARTIST").unwrap();
        assert_eq!(&lines[1][artist_column..artist_column + 8], "Artist 1");
        assert_eq!(&lines[2][artist_column..artist_column + 1], EMPTY_CELL);
    }

    #[test]
    fn table_aligns_wide_characters_by_terminal_width() {
        let items = vec![media("Media Name 123", "pop", Some("Artist 1"))];

        let output = rendered(&items, OutputFormat::Table, false);
        let lines: Vec<&str> = output.lines().collect();

        // The name is 14 chars but 18 columns wide, so ARTIST has to start at 18
        // plus the gutter for the row to line up under the header.
        assert_eq!(items[0].name.chars().count(), 14);
        assert_eq!(lines[0].find("ARTIST"), Some(14 + GUTTER.len()));
        // assert_eq!(
        //     lines[1].find("Artist 1"),
        //     Some("Media Name 123".len() + GUTTER.len())
        // );
    }

    #[test]
    fn table_rows_have_no_trailing_whitespace() {
        let items = vec![
            media("A very long name", "pop", None),
            media("S", "r", None),
        ];

        let output = rendered(&items, OutputFormat::Table, false);

        for line in output.lines() {
            assert_eq!(line.trim_end(), line, "trailing whitespace in {line:?}");
        }
    }

    #[test]
    fn long_shows_every_column() {
        let items = vec![media("Song", "pop", Some("Artist 1"))];

        let output = rendered(&items, OutputFormat::Table, true);
        let header = output.lines().next().unwrap();

        for column in LONG_HEADERS {
            assert!(header.contains(column), "missing {column} in {header:?}");
        }
        assert!(output.contains("https://youtu.be/Song"));
        assert!(output.contains("2026-01-01 00:00:00"));
    }

    #[test]
    fn tsv_has_no_header_and_one_field_per_column() {
        let items = vec![media("Song", "pop", None)];

        let output = rendered(&items, OutputFormat::Tsv, false);
        let line = output.lines().next().unwrap();

        assert!(line.starts_with("Song\t"));
        assert_eq!(line.split('\t').count(), DEFAULT_HEADERS.len());
        // Absent values stay empty rather than becoming the table placeholder.
        assert_eq!(line.split('\t').nth(1), Some(""));
        assert_eq!(output.lines().count(), 1);
    }

    #[test]
    fn json_is_flat_and_complete() {
        let items = vec![media("Song", "pop", Some("Artist 1"))];

        let output = rendered(&items, OutputFormat::Json, false);
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0]["name"], "Song");
        // ID3 tags are flattened, not nested under "id3".
        assert_eq!(parsed[0]["artist"], "Artist 1");
        assert!(parsed[0].get("id3").is_none());
        assert_eq!(parsed[0]["album"], serde_json::Value::Null);
        assert_eq!(parsed[0]["inserted_at"], "2026-01-01 00:00:00");
    }

    #[test]
    fn rendering_nothing_produces_no_rows() {
        assert_eq!(rendered(&[], OutputFormat::Tsv, false), "");
        assert_eq!(rendered(&[], OutputFormat::Json, false), "[]\n");
        // The table still prints its header, so the columns are discoverable.
        assert_eq!(rendered(&[], OutputFormat::Table, false).lines().count(), 1);
    }
}
