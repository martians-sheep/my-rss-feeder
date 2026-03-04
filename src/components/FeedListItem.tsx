import type { Feed } from "../types/feed";

interface FeedListItemProps {
  feed: Feed;
  selected: boolean;
  onClick: () => void;
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

export function FeedListItem({ feed, selected, onClick }: FeedListItemProps) {
  const label = feedTypeLabel(feed.feedType);

  return (
    <button
      onClick={onClick}
      className={`w-full rounded-lg px-3 py-2 text-left text-sm transition-colors ${
        selected
          ? "bg-gray-700 font-medium text-white"
          : "text-gray-300 hover:bg-gray-700/50 hover:text-white"
      }`}
    >
      <span className="flex items-center gap-2">
        <span className="truncate">{feed.title}</span>
        {label && (
          <span className="shrink-0 rounded bg-gray-600 px-1.5 py-0.5 text-[10px] leading-none text-gray-300">
            {label}
          </span>
        )}
      </span>
    </button>
  );
}
