# 個人用RSSリーダー 実装計画メモ

> 最終更新: 2026-03-03
> ステータス: 計画中

---

## プロジェクト概要

自分専用のRSSリーダーをデスクトップアプリとして作る。
ニュース・技術ブログ・自作RSSフィードを一元管理するのが目的。

**現状:**
- ReactでUIデモ版が完成済み（カード型レイアウト、フィード追加/削除、検索、ダーク/ライト切替）
- CORSプロキシ（corsproxy.io）経由でRSSフェッチしている
- フォントは Noto Sans JP + Source Serif 4

---

## 1. 技術スタック選定

### デスクトップフレームワーク比較

| 項目 | Electron | Tauri |
|---|---|---|
| バンドルサイズ | 大きい（150MB〜） | 小さい（10MB〜） |
| メモリ使用量 | 多い（Chromium丸ごと） | 少ない（OS WebView利用） |
| 起動速度 | やや遅い | 速い |
| Web技術との互換性 | 完全（Chromium） | ほぼ完全（WebView2/WebKit） |
| バックエンド | Node.js（JS/TS） | Rust |
| エコシステム | 成熟・情報豊富 | 成長中・v2で安定 |
| macOS/Windows対応 | 両方OK | 両方OK |
| CORSの扱い | メインプロセスでfetchすればCORS不要 | Rustサイドでfetchすれば CORS不要 |
| 自動更新 | electron-updater（実績多） | tauri-plugin-updater |

**推奨: Tauri v2**

理由:
- バンドルサイズが圧倒的に小さい → 個人用なので軽量が正義
- 起動3秒以内の非機能要件を満たしやすい
- RustサイドでHTTPリクエスト → CORSプロキシ不要になる
- 個人プロジェクトなのでRust学習コストは許容範囲
- Tauri v2は十分安定しており、プラグインエコシステムも充実

> ただし、Rustに抵抗がある場合はElectronでもOK。その場合はメインプロセスでRSSフェッチする構成にすればCORS問題は同様に解決できる。

### 主要ライブラリ選定

| カテゴリ | ライブラリ | 理由 |
|---|---|---|
| フロントエンド | React 19 + TypeScript | 既存デモの流用 |
| ビルドツール | Vite | Tauri公式テンプレートがViteベース、高速 |
| 状態管理 | Zustand | 軽量・シンプル・ボイラープレート少ない |
| RSS解析 | feed-rs（Rust crate） | Rustサイドで解析、RSS/Atom/JSON Feed対応 |
| OGP取得 | scraper（Rust crate） | HTMLパース用、Rustサイドでmeta tag抽出 |
| HTTPクライアント | reqwest（Rust crate） | Tauri公式推奨、async対応 |
| ローカルDB | SQLite（rusqlite or sqlx） | 軽量・ファイルベース・永続化に最適 |
| フロント側DB操作 | Tauri Commands経由 | Rustで定義したコマンドをフロントから呼ぶ |
| スタイリング | Tailwind CSS v4 | 既存デモで使用想定、高速開発 |
| テスト（フロント） | Vitest + Testing Library | Viteと相性◎ |
| テスト（Rust） | cargo test（標準） | Rust標準のテストフレームワーク |
| リント/フォーマット | ESLint + Prettier + clippy | フロント/Rust双方でコード品質を確保 |
| パッケージマネージャ | pnpm | 要件通り |
| ランタイム管理 | mise | 要件通り、.mise.tomlでNode/Rust固定 |

### プロジェクト構成（想定）

```
my-rss-feeder/
├── doc/                    # ドキュメント
├── src/                    # React フロントエンド
│   ├── components/
│   ├── hooks/
│   ├── stores/             # Zustand stores
│   ├── types/
│   ├── utils/
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/              # Tauri (Rust) バックエンド
│   ├── src/
│   │   ├── commands/       # Tauriコマンド定義
│   │   ├── db/             # SQLite操作
│   │   ├── feed/           # RSSフェッチ・解析
│   │   ├── ogp/            # OGP取得
│   │   └── main.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── .mise.toml
├── package.json
├── pnpm-lock.yaml
├── vite.config.ts
├── tsconfig.json
├── tailwind.config.ts
└── README.md
```

---

## 2. フェーズ分け・マイルストーン

### Phase 0: プロジェクトセットアップ（〜1週間）

**目標:** 開発環境を整え、Tauriアプリが起動する状態にする

- [ ] mise設定（.mise.toml: Node.js LTS + Rust stable）
- [ ] `pnpm create tauri-app` でプロジェクト初期化（React + TypeScript + Vite）
- [ ] Tailwind CSS v4 セットアップ
- [ ] ESLint / Prettier / clippy 設定
- [ ] 既存デモのUI部品を移植（カードレイアウト、ダーク/ライト切替）
- [ ] フォント設定（Noto Sans JP + Source Serif 4）
- [ ] CI設定（GitHub Actions: lint + test）

**完了条件:** `pnpm tauri dev` でデモUIが表示される

**優先度: 最高** — 他の全フェーズの土台

---

### Phase 1: コアフィード機能（〜2週間）

**目標:** RSSフィードの登録・取得・表示ができる

- [ ] SQLiteスキーマ設計（feeds, articles テーブル）
- [ ] Rustサイドでfeed-rs使ったRSSフェッチ・パース
- [ ] Tauriコマンド実装（add_feed, remove_feed, list_feeds, fetch_articles）
- [ ] フロントからTauriコマンド呼び出し
- [ ] フィード追加ダイアログ
- [ ] 記事一覧のカード表示（タイトル、概要、日時、フィード名）
- [ ] フィード削除機能
- [ ] RSS / Atom / JSON Feed 対応確認

**完了条件:** 複数のRSSフィードを登録し、記事がカード形式で一覧表示される

**優先度: 最高** — アプリの存在意義そのもの

---

### Phase 2: OGP取得・リッチ表示（〜1.5週間）

**目標:** 記事カードにOGP画像と説明が表示される

- [ ] Rustサイドでscraper使ったOGP取得（og:image, og:description）
- [ ] OGP情報のキャッシュ（SQLiteに保存、毎回取得しない）
- [ ] フォールバック処理
  - og:image未設定 → フィードのfaviconやプレースホルダー画像
  - og:description未設定 → RSSのdescriptionをそのまま使用
- [ ] 画像のローカルキャッシュ（ディスクに保存してオフライン対応）
- [ ] カードUIにOGP画像/説明を反映
- [ ] レート制限対策（リクエスト間隔制御、robots.txt確認はスキップ※個人用なので控えめアクセスで十分）

**完了条件:** 記事カードにOGP画像が表示され、オフラインでもキャッシュ画像が見れる

**優先度: 高** — カード表示のリッチさはUXに直結

---

### Phase 3: 既読管理・検索・フィルタ（〜1.5週間）

**目標:** 記事の既読/未読管理、検索、フィルタリングができる

- [ ] 既読/未読ステータス管理（SQLite: articles.read_at）
- [ ] 記事クリック時に既読マーク
- [ ] 未読のみ表示フィルタ
- [ ] フィードごとのフィルタ
- [ ] 全文検索（SQLite FTS5）
- [ ] 検索UIの改善（デモ版の検索を本実装に置き換え）

**完了条件:** 既読管理が機能し、検索でヒットした記事が表示される

**優先度: 高** — 日常利用に必須

---

### Phase 4: 自動更新・永続化（〜1週間）

**目標:** バックグラウンドで定期的にフィードを更新する

- [ ] バックグラウンドポーリング実装（Rustサイドでtokioタイマー）
- [ ] 更新間隔の設定UI（15分/30分/1時間/手動）
- [ ] 新着記事の通知（OS通知、tauri-plugin-notification）
- [ ] アプリ起動時のフィード設定復元確認
- [ ] ウィンドウサイズ・位置の保存/復元

**完了条件:** アプリを放置していても新着記事が自動取得される

**優先度: 高** — RSSリーダーの基本中の基本

---

### Phase 5: あれば嬉しい機能（〜2週間）

**目標:** 利便性を高める追加機能

- [ ] 未読バッジ（フィード名の横に未読件数表示）
- [ ] カテゴリ/タグ管理（フィードにタグ付け、タグで絞り込み）
- [ ] お気に入り保存（スター機能、お気に入り一覧）
- [ ] OPMLインポート/エクスポート（他のRSSリーダーとの互換）
- [ ] キーボードショートカット（j/k で記事移動、o で開く、s でスター）

**完了条件:** 各機能が動作する

**優先度: 中** — なくても使えるが、あると嬉しい

---

### Phase 6: ビルド・配布（〜1週間）

**目標:** macOS / Windows 向けにビルドして配布可能にする

- [ ] Tauriビルド設定（macOS: .dmg, Windows: .msi/.exe）
- [ ] アプリアイコン設定
- [ ] GitHub Actionsでクロスプラットフォームビルド
- [ ] GitHub Releasesに自動アップロード
- [ ] 自動更新の仕組み検討（必要なら）

**完了条件:** macOS / Windows のインストーラーが生成される

**優先度: 中** — 開発中は `tauri dev` で十分

---

## 3. リスク・課題整理

### 技術的リスク

| リスク | 影響 | 対処方針 |
|---|---|---|
| Rust学習コスト | 開発速度低下 | feed-rs/reqwest/scraper は高レベルAPI。複雑なRustは不要。困ったらElectronにフォールバック |
| Tauri v2 の不具合 | 機能制限 | GitHub Issues確認、致命的ならElectronに切替 |
| WebView の差異 | macOS/Windows で表示が違う | Tailwindベースで差異は最小限、CIで両OS確認 |

### RSSフェッチ周りの注意点

- **フィード形式の差異**: RSS 1.0 / 2.0 / Atom / JSON Feed が混在 → feed-rs が全対応するので安心
- **エンコーディング**: Shift_JIS等の日本語フィードがある → reqwestでバイト取得 → encoding_rsで変換してからパース
- **不正なXML**: 閉じタグ漏れ等 → feed-rsがある程度tolerate、ダメなら個別対応
- **リダイレクト**: フィードURLが301/302する → reqwestがデフォルトで追従
- **タイムアウト**: 応答しないサーバー → reqwestにtimeout設定（10秒程度）
- **フィード更新頻度**: ETag / Last-Modified ヘッダで304対応 → 無駄なフェッチ削減

### OGP取得周りの注意点

- **レート制限**: 一気に大量リクエストしない → セマフォで同時接続数制限（5並列程度）
- **robots.txt**: 個人用＆控えめアクセスなので厳密な対応は不要、常識的な間隔でリクエスト
- **og:image未設定**: サイトによってはOGPがない → フォールバック画像を用意
- **og:imageのURL**: 相対パスの場合がある → ベースURLで解決
- **巨大な画像**: 数MBのOGP画像 → ダウンロード時にサイズ上限設定（2MB程度）
- **タイムアウト**: OGP取得は記事表示をブロックしない → 非同期で取得、取得できたら表示更新

### その他のハマりポイント

- **SQLite同時アクセス**: Tauriのマルチスレッド環境 → r2d2やdeadpoolでコネクションプール管理
- **アプリサイズ**: Tauriは小さいが、画像キャッシュで膨らむ → 古いキャッシュの自動削除
- **macOS公証**: 配布時にApple公証が必要 → tauri-plugin-updater + Apple Developer ID（個人用なら省略可）

---

## スケジュール概要

| フェーズ | 期間目安 | 累計 |
|---|---|---|
| Phase 0: セットアップ | 1週間 | 1週間 |
| Phase 1: コアフィード | 2週間 | 3週間 |
| Phase 2: OGP | 1.5週間 | 4.5週間 |
| Phase 3: 既読/検索 | 1.5週間 | 6週間 |
| Phase 4: 自動更新 | 1週間 | 7週間 |
| Phase 5: 追加機能 | 2週間 | 9週間 |
| Phase 6: ビルド | 1週間 | 10週間 |

**合計: 約10週間（2.5ヶ月）**

> 個人開発ペースによる。Phase 4まで（7週間）で実用レベルになる。

---

## 次のアクション

1. mise + Tauri + React のプロジェクトセットアップ
2. SQLiteスキーマのドラフト作成
3. feed-rs で主要RSSフィードのパースを試す POC
