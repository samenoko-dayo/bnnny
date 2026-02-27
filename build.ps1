bun install
Invoke-WebRequest "https://github.com/yt-dlp/yt-dlp-nightly-builds/releases/latest/download/yt-dlp.exe" -OutFile ".\src-tauri\binaries\yt-dlp-x86_64-pc-windows-msvc.exe"
bun tauri build