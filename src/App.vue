<script setup lang="ts">
import { ref, onMounted, watch, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Command } from "@tauri-apps/plugin-shell";
import { open } from "@tauri-apps/plugin-dialog";
import { homeDir } from "@tauri-apps/api/path";
import { platform } from "@tauri-apps/plugin-os";
import { load } from "@tauri-apps/plugin-store";
import { Input } from "./components/ui/input";
import { Button } from "./components/ui/button";
import { Progress } from "./components/ui/progress";
import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectLabel,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import { FileVideoCamera, Folder, FileHeadphone } from "lucide-vue-next";

// 型定義
interface Log {
    type: "info" | "err" | "warn";
    msg: string;
}

interface ProfileInfo {
    name: string;
    path: string;
    is_relative: boolean;
}

interface BrowserProfiles {
    firefox: ProfileInfo[];
    floorp: ProfileInfo[];
    zen: ProfileInfo[];
}

// 状態管理
const url_input = ref("");
const deno_path = ref<string | null>(null);
const ffmpeg_path = ref<string | null>(null);
const output_path = ref("");
const log = ref<Log[]>([]);
const logArea = ref<HTMLElement | null>(null);
const downloadProgress = ref(0);
const downloadStatus = ref("");
const downloading = ref(false);

// ブラウザプロファイル
const browserProfiles = ref<BrowserProfiles | null>(null);
const selectedCookieArg = ref("");
const current_platform = platform();

// ログ系のユーティリティ
watch(
    log,
    async () => {
        await nextTick();
        if (logArea.value) {
            logArea.value.scrollTop = logArea.value.scrollHeight;
        }
    },
    { deep: true },
);

const addLog = (msg: string, type: Log["type"]) =>
    log.value.push({ type, msg });

// プロファイル取得
async function fetchProfiles() {
    try {
        const res = await invoke<BrowserProfiles>("get_all_browser_profiles");
        browserProfiles.value = res;
    } catch (error) {
        addLog(`[!] プロファイル取得失敗: ${error}`, "err");
    }
}

async function safeInvoke<T>(command: string): Promise<T | null> {
  try {
    return await invoke<T>(command);
  } catch (e) {
    console.warn(`${command} failed:`, e);
    return null;
  }
}

async function runYtDlpUpdate() {
  const updateDLP = Command.sidecar("binaries/yt-dlp", ["-U"], {
    encoding: "shift_jis",
  });

  updateDLP.stdout.on("data", (data) => {
    addLog(data.trim(), "info");
  });

  updateDLP.stderr.on("data", (data) => {
    addLog(data.trim(), "err");
  });

  addLog("[ℹ️] yt-dlpのアップデートを確認しています…", "info");

  return new Promise<void>(async (resolve, reject) => {
    try {
      updateDLP.on("close", () => {
        resolve();
      });

      await updateDLP.spawn();
    } catch (e) {
      reject(e);
    }
  });
}

onMounted(async () => {
  downloading.value = true;
  downloadStatus.value = "初期化中…";

  try {
    const store = await load("settings.json");

    const [
      denoResult,
      ffmpegResult,
      savedOutputPath,
      savedCookie,
      defaultHomeDir,
    ] = await Promise.all([
      safeInvoke<string>("get_deno_path"),
      safeInvoke<string>("get_ffmpeg_path"),
      store.get<string>("output_path"),
      store.get<string>("selectedCookieArg"),
      homeDir(),
    ]);

    deno_path.value = denoResult;
    ffmpeg_path.value = ffmpegResult;
    output_path.value = savedOutputPath || defaultHomeDir;
    selectedCookieArg.value = savedCookie || "";

    watch(output_path, async (newVal) => {
      try {
        await store.set("output_path", newVal);
        await store.save();
      } catch (e) {
        console.error("failed to save output_path:", e);
      }
    });

    watch(selectedCookieArg, async (newVal) => {
      try {
        await store.set("selectedCookieArg", newVal);
        await store.save();
      } catch (e) {
        console.error("failed to save selectedCookieArg:", e);
      }
    });

    await fetchProfiles();

    if (!deno_path.value) {
      addLog("[⚠] denoが見つかりませんでした。", "warn");
    } else {
      addLog(`[ℹ️] denoが見つかりました: ${deno_path.value}`, "info");
    }

    if (!ffmpeg_path.value) {
      addLog("[⚠] ffmpegがインストールされていません。", "warn");
    } else {
      addLog(`[ℹ️] ffmpegが見つかりました: ${ffmpeg_path.value}`, "info");
    }

    downloadStatus.value = "yt-dlpの更新確認中…";
    await runYtDlpUpdate();

    downloadStatus.value = "起動完了。";
    addLog("[ℹ️] 初期化完了", "info");
  } catch (e) {
    console.error("initialization failed:", e);
    addLog(`[❌] 初期化中にエラーが発生しました: ${String(e)}`, "err");
    downloadStatus.value = "初期化に失敗しました。";
  } finally {
    downloading.value = false;
  }
});

// 保存先の選択
const pick_savedir = async () => {
    const dir = await open({ multiple: false, directory: true });
    if (dir) {
        output_path.value = dir;
    }
};

// ダウンロード処理
const download = async (type: "audio" | "normal") => {
    if (!url_input.value) {
        addLog("URLを入力してください", "err");
        return;
    } else if (!url_input.value.startsWith("http")) {
        addLog("有効な形式ではありません", "err");
        return;
    }

    downloading.value = true;
    downloadProgress.value = 0;
    downloadStatus.value = "ダウンロードしています...";

    const ydlopts = [
        "--progress-template",
        "download:[DOWNLOADING]\t%(progress._percent_str)s\t%(info.title)s",
        "--embed-thumbnail",
        "--add-metadata"
    ];

    try {
        const urlObj = new URL(url_input.value);
        const hostname = urlObj.hostname.replace(/^www\./, "");

        const YOUTUBE_DOMAINS = new Set(["youtube.com", "youtu.be"]);
        const MUSIC_DOMAINS = new Set(["music.youtube.com"]);

        const isYoutube = YOUTUBE_DOMAINS.has(hostname);
        const isMusic = MUSIC_DOMAINS.has(hostname);
        const isPlaylist = urlObj.pathname === "/playlist" || urlObj.searchParams.has("list");

        if (isYoutube) {
            if (isPlaylist) {
                ydlopts.push("-o", `${output_path.value}/%(playlist_title)s/%(title)s.%(ext)s`)
            } else {
                ydlopts.push("-o", `${output_path.value}/%(title)s.%(ext)s`)
            }
        } else if (isMusic) {
            ydlopts.push("--convert-thumbnails", "jpg", "--ppa", "ThumbnailsConvertor:-qmin 1 -q:v 1 -vf crop=\"'if(gt(ih,iw),iw,ih)':'if(gt(iw,ih),ih,iw)'\"")
            if (isPlaylist) {
                ydlopts.push("-o", `${output_path.value}/%(album)s/%(playlist_index)02d - %(title)s.%(ext)s`, "--parse-metadata", "%(playlist_index)s/%(n_entries)s:%(track_number)s")
            } else {
                ydlopts.push("-o", `${output_path.value}/%(title)s.%(ext)s`)
            }
        } else {
            ydlopts.push("-o", `${output_path.value}/%(title)s.%(ext)s`)
        }
    } catch (e) {
        addLog("URLの解析に失敗しました", "err")
        downloading.value = false;
        return;
    }

    if (deno_path.value) {
        ydlopts.push("--js-runtimes", `deno:${deno_path.value}`);
    }

    if (ffmpeg_path.value) {
        ydlopts.push("--ffmpeg-location", ffmpeg_path.value);
    }

    if (selectedCookieArg.value && selectedCookieArg.value !== "none") {
        ydlopts.push("--cookies-from-browser", selectedCookieArg.value);
    }

    if (type === "audio") {
        ydlopts.push(
            "-f",
            "bestaudio",
            "-x",
            "--audio-format",
            "mp3",
            "--audio-quality",
            "0",
        );
    } else {
        ydlopts.push(
            "-f",
            "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best",
            "--merge-output-format",
            "mp4",
        );
    }

    ydlopts.push(url_input.value);
    addLog(`オプション: ${ydlopts.join(" ")}`, "info");

    const cmd = Command.sidecar("binaries/yt-dlp", ydlopts, {
        encoding: current_platform === "windows" ? "shift_jis" : "utf-8"
    });

    cmd.stdout.on("data", (data) => {
        const output = data.trim();
        const parts = output.split("\t");
        if (parts[0] === "[DOWNLOADING]") {
            downloadProgress.value = Number(parts[1].replace("%", ""));
            downloadStatus.value = `${parts[2].trim()}をダウンロードしています...`;
        } else {
            addLog(output, "info");
        }
    });

    cmd.stderr.on("data", (data) => addLog(data.trim(), "err"));

    cmd.on("close", (data) => {
        downloading.value = false;
        if (data.code === 0) {
            downloadProgress.value = 100;
            addLog("ダウンロードが完了しました!", "info");
            downloadStatus.value = "ダウンロードが完了しました!";
        } else {
            downloadProgress.value = 0;
            addLog(`エラーが発生しました (code: ${data.code})`, "err");
            downloadStatus.value = "ダウンロード中にエラーが発生しました";
        }
    });

    await cmd.spawn();
};
</script>

<template>
    <main class="h-screen min-h-screen flex flex-col gap-2 p-4">
        <h1 class="text-2xl font-bold">Bnnny</h1>

        <div class="flex flex-col gap-2">
            <Input v-model="url_input" placeholder="URL" />

            <div class="flex flex-row items-center gap-2">
                <Input v-model="output_path" placeholder="保存先" />
                <Button @click="pick_savedir"><Folder />選択</Button>
            </div>

            <div class="flex flex-col gap-1">
                <label class="text-sm font-medium text-gray-500">Cookieを使用するブラウザ</label>
                <Select v-model="selectedCookieArg">
                    <SelectTrigger class="w-full">
                        <SelectValue placeholder="クッキーを使用しない" />
                    </SelectTrigger>

                    <SelectContent>
                        <SelectItem value="none">使用しない</SelectItem>

                        <template v-if="browserProfiles">
                            <SelectGroup v-if="browserProfiles.firefox.length > 0">
                                <SelectLabel>Firefox</SelectLabel>
                                <SelectItem v-for="p in browserProfiles.firefox" :key="p.path"
                                    :value="`firefox:${p.path}`">
                                    {{ p.name }}
                                </SelectItem>
                            </SelectGroup>

                            <SelectGroup v-if="browserProfiles.floorp.length > 0">
                                <SelectLabel>Floorp</SelectLabel>
                                <SelectItem v-for="p in browserProfiles.floorp" :key="p.path"
                                    :value="`firefox:${p.path}`">
                                    {{ p.name }}
                                </SelectItem>
                            </SelectGroup>

                            <SelectGroup v-if="browserProfiles.zen.length > 0">
                                <SelectLabel>Zen Browser</SelectLabel>
                                <SelectItem v-for="p in browserProfiles.zen" :key="p.path" :value="`firefox:${p.path}`">
                                    {{ p.name }}
                                </SelectItem>
                            </SelectGroup>
                        </template>
                    </SelectContent>
                </Select>
            </div>
        </div>

        <div>
            <Progress :model-value="downloadProgress"></Progress>
            <p class="text-sm mt-1">{{ downloadStatus }}</p>
        </div>

        <div class="w-full flex-1 shrink overflow-y-auto font-mono p-2 rounded-sm border border-gray-200 text-xs"
            ref="logArea">
            <p v-for="log_text in log" :class="log_text.type" class="whitespace-pre-wrap">
                {{ log_text.msg }}
            </p>
        </div>

        <div class="grid grid-cols-2 gap-2">
            <Button @click="download('normal')" :disabled="downloading"><FileVideoCamera />動画+音声</Button>
            <Button @click="download('audio')" :disabled="downloading"><FileHeadphone />音声のみ</Button>
        </div>
    </main>
</template>
