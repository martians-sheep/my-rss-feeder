# 自動更新（オートアップデート）設計書

> 最終更新: 2026-03-04
> ステータス: 実装済み（署名鍵の設定・GitHub Secrets の登録は未完了）

---

## 1. 概要

My RSS Feeder デスクトップアプリに、新バージョンのリリース時にユーザーへ通知し、
アプリ内から更新を適用できる自動更新（オートアップデート）機能を導入する。

**Tauri v2 公式の `tauri-plugin-updater` を利用する。**

---

## 2. アーキテクチャ全体像

```
┌─────────────────────────────────────────────────────────────────┐
│                     GitHub 側                                    │
│                                                                  │
│  [git push tag v*]                                               │
│        │                                                         │
│        ▼                                                         │
│  GitHub Actions (publish workflow)                               │
│  ┌───────────────────────────────────────────────┐               │
│  │ matrix build (macOS/Windows/Linux)            │               │
│  │   └── tauri build + 署名 (.sig 生成)          │               │
│  │   └── latest.json 生成                        │               │
│  └───────────────┬───────────────────────────────┘               │
│                  │                                                │
│                  ▼                                                │
│  GitHub Releases                                                 │
│  ├── my-rss-feeder_0.2.0_aarch64.dmg                            │
│  ├── my-rss-feeder_0.2.0_aarch64.dmg.sig                        │
│  ├── my-rss-feeder_0.2.0_x64-setup.exe                          │
│  ├── my-rss-feeder_0.2.0_x64-setup.exe.sig                      │
│  ├── my-rss-feeder_0.2.0_amd64.AppImage                         │
│  ├── my-rss-feeder_0.2.0_amd64.AppImage.sig                     │
│  └── latest.json                                                 │
└──────────────────────────────────────────┬──────────────────────┘
                                           │
                     HTTPS GET             │
                                           │
┌──────────────────────────────────────────┼──────────────────────┐
│              デスクトップアプリ            │                      │
│                                          ▼                      │
│  ┌──────────────────────────────────────────────┐               │
│  │ tauri-plugin-updater                          │               │
│  │  1. latest.json を取得                        │               │
│  │  2. バージョン比較 (現在 vs latest)             │               │
│  │  3. 新バージョンあり → ユーザーに通知           │               │
│  │  4. ユーザー承認 → バイナリダウンロード         │               │
│  │  5. 署名検証 (公開鍵で .sig を検証)            │               │
│  │  6. インストール → アプリ再起動                │               │
│  └──────────────────────────────────────────────┘               │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. 技術選定の根拠

### なぜ `tauri-plugin-updater` を使うのか

| 方式 | メリット | デメリット |
|---|---|---|
| **tauri-plugin-updater** (採用) | Tauri公式プラグイン、署名検証付き、クロスプラットフォーム対応、GitHub Releasesとの統合が容易 | Tauri v2 専用 |
| 手動ダウンロードリンク表示 | 実装が簡単 | UXが悪い（ブラウザ経由でダウンロード→手動インストール） |
| Sparkle (macOS) / WinSparkle | OS ネイティブ、実績多 | macOS/Windows 別々に実装が必要、Tauriとの統合が面倒 |
| CrabNebula Cloud | Tauri公式チーム提供のホスティング | 有料サービス、個人プロジェクトにはオーバースペック |

**結論:** Tauri公式の `tauri-plugin-updater` + GitHub Releases が、個人プロジェクトに最もバランスが良い。

### 更新配信サーバー: GitHub Releases

- 無料、追加インフラ不要
- `tauri-action` が `latest.json` を自動生成・アップロード
- CDN (GitHub のインフラ) 経由で高速配信
- 個人プロジェクトには十分な帯域

---

## 4. 実装詳細

### 4.1 依存パッケージの追加

**Rust (`src-tauri/Cargo.toml`):**

```toml
[dependencies]
# 既存の依存に追加
tauri-plugin-updater = "2"
tauri-plugin-dialog = "2"
tauri-plugin-process = "2"
```

> `tauri-plugin-dialog`: 更新通知ダイアログ表示用
> `tauri-plugin-process`: 更新後のアプリ再起動用

**フロントエンド (`package.json`):**

```bash
pnpm add @tauri-apps/plugin-updater @tauri-apps/plugin-dialog @tauri-apps/plugin-process
```

### 4.2 Rust 側プラグイン登録 (`src-tauri/src/lib.rs`)

```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())   // 追加
        .plugin(tauri_plugin_process::init())   // 追加
        .setup(|app| {
            // 既存のセットアップ処理 ...

            // アップデータープラグインの登録
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 既存のコマンド ...
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 4.3 Tauri 設定 (`tauri.conf.json`)

```jsonc
{
  "productName": "My RSS Feeder",
  "version": "0.2.0",
  "identifier": "com.myrssfeed.app",
  "bundle": {
    "active": true,
    "targets": "all",
    "createUpdaterArtifacts": true,    // ← 追加: 署名付きアーティファクト生成
    "icon": [...]
  },
  "plugins": {                          // ← 追加
    "updater": {
      "pubkey": "<署名用公開鍵をここに設定>",
      "endpoints": [
        "https://github.com/<user>/my-rss-feeder/releases/latest/download/latest.json"
      ]
    }
  }
  // ... その他の設定
}
```

### 4.4 権限設定 (`src-tauri/capabilities/default.json`)

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default permissions for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open",
    "updater:default",
    "dialog:default",
    "process:allow-restart"
  ]
}
```

### 4.5 署名鍵の生成

```bash
# 鍵ペア生成（パスワードの設定を求められる）
pnpm tauri signer generate -w ~/.tauri/my-rss-feeder.key
```

生成されるファイル:
- `~/.tauri/my-rss-feeder.key` — 秘密鍵（**絶対に公開しない**）
- `~/.tauri/my-rss-feeder.key.pub` — 公開鍵（`tauri.conf.json` の `pubkey` に設定）

**重要:**
- 秘密鍵はパスワードマネージャー等で安全に保管
- 秘密鍵を紛失すると、既存ユーザーへの更新配信が不可能になる
- GitHub Actions の Secrets に以下を登録:
  - `TAURI_SIGNING_PRIVATE_KEY`: 秘密鍵の内容
  - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: 秘密鍵のパスワード

### 4.6 フロントエンド更新チェック UI

**方針:** アプリ起動時にバックグラウンドで更新を確認し、新バージョンがあればダイアログで通知する。

```typescript
// src/hooks/useAppUpdater.ts
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { ask } from '@tauri-apps/plugin-dialog';
import { useEffect } from 'react';

export function useAppUpdater() {
  useEffect(() => {
    checkForUpdates();
  }, []);
}

async function checkForUpdates() {
  try {
    const update = await check();
    if (!update) return; // 最新バージョン

    const shouldUpdate = await ask(
      `新しいバージョン ${update.version} が利用可能です。\n\n` +
      `更新内容:\n${update.body ?? '(詳細なし)'}\n\n` +
      `今すぐ更新しますか？`,
      {
        title: 'アップデートのお知らせ',
        kind: 'info',
        okLabel: '更新する',
        cancelLabel: '後で',
      }
    );

    if (shouldUpdate) {
      await update.downloadAndInstall();
      await relaunch();
    }
  } catch (error) {
    console.error('更新チェックに失敗しました:', error);
    // エラーは静かに無視（ネットワーク未接続時など）
  }
}
```

**App.tsx での使用:**

```tsx
import { useAppUpdater } from './hooks/useAppUpdater';

function App() {
  useAppUpdater();  // アプリ起動時に更新チェック
  // ... 既存のレンダリング
}
```

### 4.7 latest.json の構造（参考）

`tauri-action` が自動生成する。手動で作成する必要はない。

```json
{
  "version": "0.2.0",
  "notes": "バグ修正と新機能の追加",
  "pub_date": "2026-03-04T12:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "<署名内容>",
      "url": "https://github.com/<user>/my-rss-feeder/releases/download/v0.2.0/my-rss-feeder_0.2.0_aarch64.app.tar.gz"
    },
    "darwin-x86_64": {
      "signature": "<署名内容>",
      "url": "https://github.com/<user>/my-rss-feeder/releases/download/v0.2.0/my-rss-feeder_0.2.0_x64.app.tar.gz"
    },
    "linux-x86_64": {
      "signature": "<署名内容>",
      "url": "https://github.com/<user>/my-rss-feeder/releases/download/v0.2.0/my-rss-feeder_0.2.0_amd64.AppImage"
    },
    "windows-x86_64": {
      "signature": "<署名内容>",
      "url": "https://github.com/<user>/my-rss-feeder/releases/download/v0.2.0/my-rss-feeder_0.2.0_x64-setup.exe"
    }
  }
}
```

---

## 5. CI/CD: GitHub Actions ワークフロー

### 5.1 リリースワークフロー (`.github/workflows/publish.yml`)

```yaml
name: Publish Release

on:
  push:
    tags:
      - 'v*'  # v0.2.0, v1.0.0 等のタグで発火

jobs:
  publish-tauri:
    permissions:
      contents: write  # リリース作成に必要

    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: 'macos-latest'
            args: '--target aarch64-apple-darwin'
          - platform: 'macos-latest'
            args: '--target x86_64-apple-darwin'
          - platform: 'ubuntu-22.04'
            args: ''
          - platform: 'windows-latest'
            args: ''

    runs-on: ${{ matrix.platform }}

    steps:
      - uses: actions/checkout@v4

      # Node.js セットアップ
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: lts/*

      # pnpm セットアップ
      - name: Setup pnpm
        uses: pnpm/action-setup@v4
        with:
          version: latest
          run_install: false

      # Rust セットアップ
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.platform == 'macos-latest' && 'aarch64-apple-darwin,x86_64-apple-darwin' || '' }}

      # Rust キャッシュ
      - name: Rust cache
        uses: swatinem/rust-cache@v2
        with:
          workspaces: './src-tauri -> target'

      # Linux 依存パッケージ
      - name: Install Linux dependencies
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev \
            libappindicator3-dev \
            librsvg2-dev \
            patchelf

      # フロントエンド依存インストール
      - name: Install frontend dependencies
        run: pnpm install

      # Tauri ビルド & リリース
      - name: Build and release
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        with:
          tagName: v__VERSION__
          releaseName: 'My RSS Feeder v__VERSION__'
          releaseBody: 'リリースノートはこちらを参照してください。'
          releaseDraft: true
          prerelease: false
          args: ${{ matrix.args }}
```

### 5.2 リリース手順

```bash
# 1. バージョンを更新（3箇所を同期させる）
#    - src-tauri/tauri.conf.json  → "version": "0.2.0"
#    - src-tauri/Cargo.toml       → version = "0.2.0"
#    - package.json               → "version": "0.2.0"

# 2. コミット & タグ
git add -A
git commit -m "release: v0.2.0"
git tag v0.2.0

# 3. プッシュ（GitHub Actions が自動でビルド & リリース作成）
git push origin main --tags

# 4. GitHub でドラフトリリースを確認し、公開
```

---

## 6. セキュリティ考慮事項

| 項目 | 対策 |
|---|---|
| バイナリ改ざん防止 | Ed25519 署名で検証。公開鍵をアプリにバンドルし、ダウンロード後に `.sig` ファイルで検証 |
| 中間者攻撃 | HTTPS 通信のみ（GitHub Releases は HTTPS 強制） |
| 秘密鍵の管理 | GitHub Actions Secrets に保存。リポジトリにはコミットしない |
| ダウングレード攻撃 | バージョン比較により、現在より古いバージョンへは更新しない |

---

## 7. UX 設計

### 更新フロー（ユーザー視点）

```
アプリ起動
    │
    ├── バックグラウンドで latest.json を確認
    │       │
    │       ├── 最新バージョン → 何も表示しない
    │       │
    │       └── 新バージョンあり
    │               │
    │               ▼
    │         ┌─────────────────────────────┐
    │         │ アップデートのお知らせ         │
    │         │                              │
    │         │ 新しいバージョン 0.2.0 が     │
    │         │ 利用可能です。               │
    │         │                              │
    │         │ 更新内容:                     │
    │         │ - バグ修正                    │
    │         │ - 新機能追加                  │
    │         │                              │
    │         │ [更新する]    [後で]          │
    │         └─────────────────────────────┘
    │               │               │
    │               ▼               └── アプリ通常起動
    │         ダウンロード中...
    │         (プログレス表示)
    │               │
    │               ▼
    │         インストール完了
    │         アプリ自動再起動
    │
    └── アプリ通常起動
```

### 設計方針

- **非ブロッキング:** 更新チェックはバックグラウンドで行い、アプリの起動を妨げない
- **ユーザー主導:** 更新の適用はユーザーの明示的な承認が必要（強制更新しない）
- **失敗に寛容:** ネットワークエラー時はエラーを握りつぶし、次回起動時に再チェック
- **日本語UI:** ダイアログは日本語で表示

---

## 8. プラットフォーム別の注意事項

### macOS

- **コード署名:** Apple Developer ID による署名がないと Gatekeeper で警告が出る
  - 個人利用なら「このまま開く」で回避可能
  - 配布するなら Apple Developer Program ($99/年) への加入が必要
- **公証 (Notarization):** macOS 10.15+ では公証が推奨
  - tauri-action が `APPLE_CERTIFICATE` 等の環境変数で自動対応可能
- **更新形式:** `.app.tar.gz` をダウンロード → 展開 → 既存アプリを置き換え

### Windows

- **コード署名:** 署名なしでも動作するが SmartScreen 警告が表示される
  - コード署名証明書は有料（年数万円〜）
  - 個人利用なら警告を無視して続行可能
- **更新形式:** NSIS インストーラー (`.exe`) をダウンロード → サイレントインストール
- **管理者権限:** 更新のインストール時に UAC プロンプトが表示される場合がある

### Linux

- **更新形式:** AppImage をダウンロード → 既存ファイルを置き換え
- **コード署名:** 不要（Tauri の Ed25519 署名で十分）
- **制約:** `.deb` パッケージは自動更新非対応。AppImage を推奨

---

## 9. 実装ステップ（チェックリスト）

### Step 1: 署名鍵の準備
- [ ] `pnpm tauri signer generate` で鍵ペアを生成
- [ ] 公開鍵を `tauri.conf.json` の `plugins.updater.pubkey` に設定
- [ ] 秘密鍵とパスワードを安全に保管

### Step 2: 依存パッケージの追加
- [x] `Cargo.toml` に `tauri-plugin-updater`, `tauri-plugin-dialog`, `tauri-plugin-process` を追加
- [x] `pnpm add @tauri-apps/plugin-updater @tauri-apps/plugin-dialog @tauri-apps/plugin-process`

### Step 3: Tauri 設定の更新
- [x] `tauri.conf.json` に `plugins.updater` セクションを追加
- [x] `tauri.conf.json` の `bundle.createUpdaterArtifacts` を `true` に設定
- [x] `capabilities/default.json` に権限を追加

### Step 4: Rust 側の実装
- [x] `lib.rs` にプラグインを登録

### Step 5: フロントエンドの実装
- [x] `useAppUpdater` フックを作成
- [x] `App.tsx` でフックを使用

### Step 6: GitHub Actions の設定
- [x] `.github/workflows/publish.yml` を作成
- [ ] GitHub Secrets に署名鍵を登録
- [ ] リポジトリの Actions 権限を「Read and write」に設定

### Step 7: テスト & 検証
- [ ] ローカルビルドで署名付きアーティファクトが生成されることを確認
- [ ] タグプッシュでGitHub Actions が正常に動作することを確認
- [ ] 実際にバージョンを上げて自動更新が動作することを確認

---

## 10. 将来の拡張（オプション）

- **更新チャンネル:** stable / beta の切替（`tauri-plugin-updater` の `headers` オプションで実現可能）
- **進捗表示:** ダウンロード進捗をプログレスバーで表示（`update.downloadAndInstall()` のコールバックで取得可能）
- **更新チェック間隔:** アプリ起動時だけでなく、定期的にチェック（例: 6時間ごと）
- **リリースノート表示:** 更新ダイアログにマークダウンのリリースノートをリッチ表示
- **ロールバック:** 問題発生時に前バージョンに戻す仕組み

---

## 参考リンク

- [Tauri v2 Updater Plugin 公式ドキュメント](https://v2.tauri.app/plugin/updater/)
- [tauri-action (GitHub Actions)](https://github.com/tauri-apps/tauri-action)
- [Tauri v2 GitHub パイプライン](https://v2.tauri.app/distribute/pipelines/github/)
- [tauri-plugin-updater リポジトリ](https://github.com/tauri-apps/tauri-plugin-updater)
- [@tauri-apps/plugin-updater JS API リファレンス](https://v2.tauri.app/reference/javascript/updater/)
