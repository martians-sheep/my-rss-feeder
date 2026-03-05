# プロジェクトルール

## コミットメッセージ

- 原則として日本語で記述する
- prefixは英語（feat, fix, refactor, docs, test, chore など）を使用する

## 言語ルール

- PR のタイトル・本文は日本語で記述する
- コード内のコメントは日本語で記述する

## 開発コマンド

- `pnpm dev` — フロントエンド開発サーバー起動（port 1420）
- `pnpm tauri dev` — Tauriアプリ起動（フロントエンド+バックエンド）
- `pnpm build` — フロントエンドビルド（tsc + vite build）
- `pnpm tauri build` — リリースビルド（.app / .dmg 生成）
- `pnpm test` — フロントエンドテスト（vitest）
- `pnpm lint` — ESLintチェック
- `cd src-tauri && cargo test` — Rustユニットテスト
- `cd src-tauri && cargo check` — Rustコンパイルチェック

## 技術スタック

- **フロントエンド**: React 19 + TypeScript + Zustand + Tailwind CSS 4
- **バックエンド**: Tauri v2 + Rust + SQLite（rusqlite bundled）+ tokio
- **パッケージマネージャ**: pnpm

## プロジェクト構造

```text
src/                          # フロントエンド
├── components/               # Reactコンポーネント（PascalCase.tsx）
├── stores/                   # Zustandストア（camelCase.ts）
├── types/                    # TypeScript型定義
├── hooks/                    # カスタムフック（useXxx.ts）
└── test/                     # テストセットアップ

src-tauri/src/                # バックエンド（Rust）
├── models/                   # データモデル（serde serialize/deserialize）
├── db/                       # DBレイヤー（migrations, *_repo.rs）
├── feed/                     # フィード取得・解析
├── ogp/                      # OGPメタデータ取得・画像キャッシュ
├── notification/             # 通知スケジューラ
├── commands.rs               # Tauriコマンドハンドラ
├── error.rs                  # AppErrorエラー型
└── lib.rs                    # アプリ初期化・プラグイン登録
```

## コーディング規約

### Rust

- エラーハンドリング: `AppError`（thiserror）を一貫して使用。コマンドは `Result<T, AppError>` を返す
- DB状態: `Arc<Database>` を Tauri State で管理。コマンドでは `State<'_, std::sync::Arc<Database>>`
- シリアライゼーション: `#[serde(rename_all = "camelCase")]` でJSON出力はcamelCase
- DBリポジトリ: エンティティごとに `*_repo.rs` を作成、関数は `&Connection` を受け取る
- テスト: 各モジュール内に `#[cfg(test)] mod tests` でインメモリSQLiteを使用

### TypeScript / React

- ストアパターン: Zustand の `create<State>()` + `invoke()` で Tauri コマンド呼び出し
- コンポーネント: 関数コンポーネント + hooks、Props インターフェースを定義
- スタイリング: Tailwind CSS のユーティリティクラスを直接使用
- Prettier: ダブルクォート、セミコロンあり、trailingComma: all

### 共通

- UI表示テキストは日本語
- ファイル名: Rust は snake_case、コンポーネントは PascalCase、それ以外のTSは camelCase
- Tauri コマンド名は snake_case（`add_feed`, `list_feeds` など）
