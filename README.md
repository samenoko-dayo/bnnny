# bnnny
## About
bnnnyは「簡単に使える」を目標に開発しているyt-dlpのGUIフロントエンドです。  
設定項目を減らし、パッと使えます。

## 機能一覧
### ブラウザプロファイルの自動取得
Firefox,Floorp,Zen Browserのプロファイルを自動で取得し、簡単に選択できます。  
これはCookieの読み込みに使用できます。

### URLによるオプションの自動選択
URLを解析し自動でオプションを選択します。

- 共通  
  サムネイルの埋め込み,メタデータの追加

- YouTube(`youtube.com`,`youtu.be`)
  - プレイリストの場合  
    ファイル命名のルールを`%(playlist_title)s/%(title)s.%(ext)s`に。

  - プレイリストでない場合  
    ファイル命名のルールを`%(title)s.%(ext)s`に。
 
- YouTube Music(`music.youtube.com`)  
  - 共通  
    サムネイルを1:1にクロップ

  - プレイリストの場合  
    ファイル命名のルールを`%(album)s/%(playlist_index)02d - %(title)s.%(ext)s`に。
 
  - プレイリストでない場合  
    ファイル命名のルールを`%(title)s.%(ext)s`に。

## 対応サイト
YouTube, YouTube Musicを対象にしています。他のサイトでは動作しない場合があります。

## 動作に必要なもの
### ffmpeg
動画と音声をマージ、動画の変換などにffmpegを必要とします。  
パッケージマネージャなどを用いffmpegをインストールしてください。

### deno
チャレンジの解決にJavaScriptのランタイムを必要とします。denoが推奨されているため、インストールしてください。  
https://deno.com を参考にインストールしてください。

## 開発環境

### Windows
- Windows 11 Pro
- Visual Studio Code

### macOS
- MacBook Air (2020,M1)
- macOS 15.7.4
- Gppgle Antigravity