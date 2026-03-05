# 日付フィルタリング: SQLite文字列比較の不一致バグ

## 発見日

2026-03-04

## ステータス

未修正

## 概要

日付フィルタで `from` と `to` を同じ日付（例: 2026/03/03）に設定すると、その日の記事が表示されない。
SQLiteの文字列比較において、DB内の日付フォーマット（`+00:00`）とフロントエンドが生成するフィルター値（`Z`）の不一致が原因。

## 根本原因

DB内の日付とJavaScriptが生成するフィルター値で、タイムゾーン表記が異なる。

| レイヤー | フォーマット | 例 |
|---------|------------|---|
| `published_at` (Rust feed_rs `to_rfc3339()`) | `+00:00`, ミリ秒なし | `2026-03-03T23:30:00+00:00` |
| `created_at` (Rust chrono `to_rfc3339()`) | `+00:00`, マイクロ秒あり | `2026-03-03T22:53:40.772996+00:00` |
| フィルター値 (JS `toISOString()`) | `Z`, ミリ秒あり | `2026-03-03T00:00:00.000Z` |

SQLiteは日付型を持たず、`>=` / `<` 比較はバイト単位の辞書順（ASCII順）で行われる。

### ASCII順の問題

```
'+' = ASCII 43
'.' = ASCII 46
'Z' = ASCII 90
```

秒の直後の文字で比較が決まるケースがあり、`+00:00` 形式の値が `Z` 形式より常に「小さい」と判定される。

### 具体例

```
DB値:       2026-03-03T23:30:00+00:00
フィルター: 2026-03-04T00:00:00.000Z  (dateTo, 排他的上限)

比較位置19: '+' (43) vs '.' (46)
結果: DB値 < フィルター値 → OK（この例ではたまたま正しい）

DB値:       2026-03-03T00:00:00+00:00
フィルター: 2026-03-03T00:00:00.000Z  (dateFrom, 以上)

比較位置19: '+' (43) vs '.' (46)
結果: DB値 < フィルター値 → dateFrom以上の条件を満たさない！（バグ）
```

### 実測データ

同一条件でフォーマットのみ変えた結果:
- `Z` 形式でフィルタ → **102件**
- `+00:00` 形式でフィルタ → **111件**
- **9件が境界条件で誤って除外**

## 影響範囲

- 日付フィルタの全プリセット（今日・過去7日・過去30日）
- カスタム日付範囲指定
- 境界付近の記事が表示されない（特に `dateFrom` 条件で顕著）

## 関連コード

- `src/stores/feedStore.ts` — `computeDateRange()` 関数（`toISOString()` で `Z` 形式を生成）
- `src-tauri/src/feed/parser.rs:72-75` — `published_at` の生成（`to_rfc3339()` で `+00:00` 形式）
- `src-tauri/src/commands.rs:40` — `created_at` の生成（`chrono::Utc::now().to_rfc3339()`）
- `src-tauri/src/db/article_repo.rs` — `list_articles()` のSQL WHERE句

## 修正案

### 案A: フロントエンド側を修正（最小影響）

`computeDateRange` で `toISOString()` の `Z` を `+00:00` に置換:

```typescript
function toRfc3339(date: Date): string {
  return date.toISOString().replace("Z", "+00:00");
}
```

### 案B: Rust側を修正（根本対応）

全ての日付を `Z` 形式で統一保存:

```rust
use chrono::SecondsFormat;
let now = chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
// => "2026-03-03T03:43:19.365Z"
```

feed_rs の `DateTime` についても `Z` 形式に正規化。既存データのマイグレーションも必要。

### 案C: SQL側を修正（データ変更不要）

```sql
WHERE REPLACE(COALESCE(a.published_at, a.created_at), '+00:00', 'Z') >= ?
```

### 推奨

案Aが最もシンプルで影響範囲が小さい。長期的には案Bでデータフォーマットを統一するのが望ましい。
