import { SidebarHeader } from "./SidebarHeader";
import { FeedList } from "./FeedList";
import { AddFeedButton } from "./AddFeedButton";

interface SidebarProps {
  onAddFeed: () => void;
  onOpenSettings: () => void;
}

export function Sidebar({ onAddFeed, onOpenSettings }: SidebarProps) {
  return (
    <aside className="flex w-64 shrink-0 flex-col bg-gray-800">
      <SidebarHeader onOpenSettings={onOpenSettings} />
      <div className="flex-1 overflow-y-auto px-3">
        <FeedList />
      </div>
      <div className="p-3">
        <AddFeedButton onClick={onAddFeed} />
      </div>
    </aside>
  );
}
