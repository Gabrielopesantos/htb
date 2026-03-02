# htb

A command-line tool for downloading and managing audio content from YouTube videos. It uses `yt-dlp` and `ffmpeg` to extract high-quality audio (MP3) and maintains a local catalog of your downloads for easy tracking and management.

## Features

- **Download**: Download audio from YouTube videos and automatically record metadata in a local catalog
- **Record**: Record video metadata without downloading the audio file
- **List**: Browse your catalog of downloaded or recorded media
- **Diff**: Download audio for previously recorded videos that are missing locally
- **Organize**: Store media in custom directories and tag them for better organization
- **SQLite Catalog**: Persistent storage of media metadata using SQLite

## Usage

### Download Audio

Download audio from a YouTube video and add it to your catalog:

```bash
htb download -u "https://www.youtube.com/watch?v=VIDEO_ID"
```

### Download to Specific Directory

```bash
htb download -u "https://www.youtube.com/watch?v=VIDEO_ID" -d "music/rock"
```

### Download with Custom Filename

```bash
htb download -u "https://www.youtube.com/watch?v=VIDEO_ID" -f "my_song.mp3"
```

### Record Metadata Only

Record video information without downloading:

```bash
htb record -u "https://www.youtube.com/watch?v=VIDEO_ID"
```

### List Catalog

List all media in your catalog:

```bash
htb list
```

List media in a specific directory:

```bash
htb list -d "music"
```

### Download Missing Files

Download audio for all recorded videos that don't have local files:

```bash
htb diff
```

## Configuration

htb uses a JSON configuration file located at `~/.config/htb/config.json`. The configuration includes:

- `catalog_path`: Directory where media files and the catalog database are stored (default: `/tmp/htb`)
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

## Dependencies

- **yt-dlp**: For downloading and extracting audio from YouTube
- **ffmpeg**: For audio conversion and processing

## License

MIT
