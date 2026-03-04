# RSS削除機能 実装計画

## 現状分析

バックエンドとストアの削除ロジックは**既に実装済み**：
- **Rust**: `feed_repo::delete_feed()` + `commands::remove_feed` (カスケード削除でarticlesも自動削除)
- **Store**: `feedStore.removeFeed()` (invoke + state更新 + selectedFeedIdリセット済み)
- **テスト**: バックエンド・ストア両方にテスト済み

**不足しているのはUIのみ。**

## 実装ステップ

### ステップ1: 削除確認ダイアログの作成
**ファイル**: `src/components/DeleteFeedDialog.tsx` (新規)

- `AddFeedDialog` と同じモーダルパターンを踏襲
- Props: `open`, `onClose`, `feed` (削除対象のFeed)
- フィード名を表示して「このフィードを削除しますか？」と確認
- 「キャンセル」「削除」ボタン
- 削除ボタン押下で `removeFeed(feed.id)` → `loadArticles()` → `onClose()`

### ステップ2: FeedListItemに削除ボタンを追加
**ファイル**: `src/components/FeedListItem.tsx` (既存)

- ホバー時に右端にゴミ箱アイコン（×ボタン）を表示
- クリックで `onDelete(feed)` コールバックを呼び出し
- フィード選択クリックと削除ボタンクリックのイベント伝播を `stopPropagation` で分離

### ステップ3: FeedListでダイアログの状態管理
**ファイル**: `src/components/FeedList.tsx` (既存)

- `DeleteFeedDialog` を組み込み
- `deletingFeed` state で削除対象フィードを管理
- `FeedListItem` の `onDelete` コールバックでダイアログを開く

### ステップ4: テストの追加
**ファイル**:
- `src/components/__tests__/DeleteFeedDialog.test.tsx` (新規)
- `src/components/__tests__/FeedList.test.tsx` (既存に追加)

テスト内容:
- DeleteFeedDialog: 表示/非表示、フィード名の表示、キャンセル動作、削除実行
- FeedList: 削除ボタンのホバー表示、削除フロー全体

## 変更ファイル一覧

| ファイル | 変更種別 |
|---------|---------|
| `src/components/DeleteFeedDialog.tsx` | 新規作成 |
| `src/components/FeedListItem.tsx` | 修正（削除ボタン追加） |
| `src/components/FeedList.tsx` | 修正（ダイアログ連携） |
| `src/components/__tests__/DeleteFeedDialog.test.tsx` | 新規作成 |
| `src/components/__tests__/FeedList.test.tsx` | 修正（削除テスト追加） |

## 変更が不要なもの

- Rustバックエンド（`remove_feed`コマンド実装済み）
- Zustandストア（`removeFeed`メソッド実装済み）
- データベース（カスケード削除設定済み）
