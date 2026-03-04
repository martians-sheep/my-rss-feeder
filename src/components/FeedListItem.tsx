import type { Feed } from "../types/feed";

interface FeedListItemProps {
  feed: Feed;
  selected: boolean;
  onClick: () => void;
  onDelete: (feed: Feed) => void;
}

/** フィードタイプの表示ラベルを返す */
function feedTypeLabel(feedType: string | null): string | null {
  switch (feedType) {
    case "atom":
      return "Atom";
    case "rss2":
      return "RSS";
    case "rss1":
      return "RSS1";
    case "rss0":
      return "RSS0";
    case "json":
      return "JSON";
    default:
      return null;
  }
}

export function FeedListItem({
  feed,
  selected,
  onClick,
  onDelete,
}: FeedListItemProps) {
  const label = feedTypeLabel(feed.feedType);

  return (
    <button
      onClick={onClick}
      className={`group relative w-full rounded-lg px-3 py-2 text-left text-sm transition-colors ${
        selected
          ? "bg-gray-700 font-medium text-white"
          : "text-gray-300 hover:bg-gray-700/50 hover:text-white"
      }`}
    >
      <span className="flex items-center gap-2 pr-6">
        <span className="truncate">{feed.title}</span>
        {label && (
          <span className="shrink-0 rounded bg-gray-600 px-1.5 py-0.5 text-[10px] leading-none text-gray-300">
            {label}
          </span>
        )}
      </span>
      <span
        role="button"
        aria-label={`${feed.title}を削除`}
        onClick={(e) => {
          e.stopPropagation();
          onDelete(feed);
        }}
        className="absolute top-1/2 right-2 -translate-y-1/2 rounded p-0.5 text-gray-400 opacity-0 transition-opacity hover:text-red-400 group-hover:opacity-100"
      >
        ✕
      </span>
    </button>
  );
}
