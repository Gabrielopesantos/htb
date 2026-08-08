# htb

A command-line tool for downloading and managing audio content from YouTube. It uses `yt-dlp` and `ffmpeg` to extract high-quality audio (MP3) and maintains a local catalog of your downloads for easy tracking and management.

## Features

- **Download**: Download audio from YouTube and automatically record metadata in a local catalog
- **Record**: Record audio metadata without downloading the audio file
- **List**: Browse your catalog of downloaded or recorded audio
- **Diff**: Download audio for previously recorded entries that are missing locally
- **ID3 Tagging**: Files get metadata and cover art from the source automatically, with optional explicit overrides
- **Tag**: Fix the tags of already-downloaded audio, in both the file and the catalog
- **Organize**: Store audio in custom directories and tag it for better organization
- **SQLite Catalog**: Persistent storage of audio metadata using SQLite

## Usage

### Download Audio

Download audio from a YouTube URL and add it to your catalog:

```bash
htb download -u "https://www.youtube.com/watch?v=<VIDEO_ID>"
```

### Download to Specific Directory

```bash
htb download -u "https://www.youtube.com/watch?v=<VIDEO_ID>" -d "<DIRECTORY>"
```

### Download with Custom Filename

```bash
htb download -u "https://www.youtube.com/watch?v=<VIDEO_ID>" -f "<FILENAME>"
```

### ID3 Tags

Every download embeds metadata and cover art taken from the source. For a YouTube Music URL that is usually correct; for a plain YouTube URL there is no artist field, so it falls back to the uploader.

Override any of it explicitly:

```bash
htb download -u "https://youtu.be/<VIDEO_ID>" -d "<DIRECTORY>" \
  --title "<TITLE>" \
  --artist "<ARTIST>" \
  --album "<ALBUM>" \
  --track <TRACK> --year <YEAR> --genre "<GENRE>"
```

Overrides are saved in the catalog, so `diff` re-applies them when it restores a missing file.

> **Note:** `-t/--tags` are catalog labels used for filtering with `htb list`. They are unrelated to `--genre`, which is the ID3 genre written into the file.

### Title Cleanup

Promotional noise is dropped automatically from the ID3 title, the catalog name, and the filename:

### Fixing Tags Afterwards

Re-running `download` on something already downloaded does nothing - the download archive skips it, so new `--artist`/`--genre` flags would be ignored. Use `tag` instead, which updates both the file and the catalog:

```bash
htb tag -u "https://youtu.be/<VIDEO_ID>" --artist "<ARTIST>" --genre "<GENRE>"
```

Only the fields you pass change, so a later `htb tag -u ... --album "<ALBUM>"` keeps the artist and genre set earlier. If the file is missing the catalog is still updated, and `htb diff` applies the tags when it restores the file.

### Record Metadata Only

Record audio information without downloading:

```bash
htb record -u "https://www.youtube.com/watch?v=<VIDEO_ID>"
```

### List Catalog

List all audio in your catalog, as an aligned table sorted by directory then name:

```bash
htb list
```

Filter by directory, or by catalog labels (`-t` matches the labels set with `--tags`, not the ID3 genre):

```bash
htb list -d "<DIRECTORY PATH>"
htb list -t "<TAG>"
```

Show every column - track, year, genre, filename, URL and the date the entry was added:

```bash
htb list --long
```

Sort by `name`, `artist`, `album`, `library` (default) or `added`, optionally reversed. Entries missing the
sort key are always listed last:

```bash
htb list --sort added --reverse   # most recently added first
```

For scripting, `--format tsv` prints the same columns without a header or padding, and `--format json`
prints every field regardless of `--long`:

```bash
htb list --format tsv | cut -f1
htb list --format json | jq -r '.[] | select(.artist == null) | .name'
```

Only the rows go to stdout; counts, warnings and the "No items to list" message go to stderr, so piping
stays clean.

### Download Missing Files

Download audio for all recorded entries that don't have local files:

```bash
htb diff
```

## Configuration

htb uses a JSON configuration file located at `~/.config/htb/config.json` by default. The configuration includes:

- `catalog_path`: Directory where audio files and the catalog database are stored (default: `~/Music/htb` with fallback to `~/htb`)
- `no_record`: If `true`, disables catalog recording (default: `false`)
- `override_if_exists`: If `true`, overwrites existing files when downloading (default: `false`)

Example configuration:

```json
{
  "catalog_path": "/home/user/music",
  "no_record": false,
  "override_if_exists": false
}
```

If the config file doesn't exist, it will be created with default values on first run.

Run `htb config` to print the path htb is using along with its current contents.

### Overriding the config location

The config file path can be overridden, in order of precedence:

1. `--config <PATH>` flag
2. `HTB_CONFIG` environment variable
3. default OS config directory (`~/.config/htb/config.json` on Linux)

## Dependencies

- **yt-dlp**: For downloading and extracting audio from YouTube
- **ffmpeg**: For audio conversion and processing

## License

MIT
