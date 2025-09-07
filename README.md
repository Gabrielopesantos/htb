# htb

## Description

Download and keep track of audio content from youtube

## Features

- `download` - Download and record audio content from youtube videos;
- `record` - Record video metadata without downloading it;
- `list` - List all persisted video metadata;
- `diff` - Download recorded audio content from persisted records whose audio isn't available locally;

## TODO

- Allow `--no-record` to skip recording metadata
- If no commands are passed in, display the help message

## Dependencies

- yt-dlp
- ffmpeg

## License

MIT
