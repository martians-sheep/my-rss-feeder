import { useEffect, useState } from "react";
import { useFeedStore } from "./stores/feedStore";
import { Sidebar } from "./components/Sidebar";
import { MainContent } from "./components/MainContent";
import { AddFeedDialog } from "./components/AddFeedDialog";

function App() {
  const [dialogOpen, setDialogOpen] = useState(false);
  const loadFeeds = useFeedStore((s) => s.loadFeeds);
  const loadArticles = useFeedStore((s) => s.loadArticles);

  useEffect(() => {
    loadFeeds();
    loadArticles();
  }, [loadFeeds, loadArticles]);

  return (
    <div className="flex h-screen bg-gray-50">
      <Sidebar onAddFeed={() => setDialogOpen(true)} />
      <MainContent />
      <AddFeedDialog open={dialogOpen} onClose={() => setDialogOpen(false)} />
    </div>
  );
}

export default App;
