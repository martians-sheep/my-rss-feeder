import { useEffect, useState } from "react";
import { useFeedStore } from "./stores/feedStore";
import { useSettingsStore } from "./stores/settingsStore";
import { Sidebar } from "./components/Sidebar";
import { MainContent } from "./components/MainContent";
import { AddFeedDialog } from "./components/AddFeedDialog";
import { SettingsDialog } from "./components/SettingsDialog";

function App() {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const loadFeeds = useFeedStore((s) => s.loadFeeds);
  const loadArticles = useFeedStore((s) => s.loadArticles);
  const loadSettings = useSettingsStore((s) => s.loadSettings);

  useEffect(() => {
    loadFeeds();
    loadArticles();
    loadSettings();
  }, [loadFeeds, loadArticles, loadSettings]);

  return (
    <div className="flex h-screen bg-gray-50">
      <Sidebar
        onAddFeed={() => setDialogOpen(true)}
        onOpenSettings={() => setSettingsOpen(true)}
      />
      <MainContent />
      <AddFeedDialog open={dialogOpen} onClose={() => setDialogOpen(false)} />
      <SettingsDialog
        open={settingsOpen}
        onClose={() => setSettingsOpen(false)}
      />
    </div>
  );
}

export default App;
