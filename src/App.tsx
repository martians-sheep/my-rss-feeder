import { useEffect, useState } from "react";
import { useFeedStore } from "./stores/feedStore";
import { useSettingsStore } from "./stores/settingsStore";
import { useArticleViewStore } from "./stores/articleViewStore";
import { Sidebar } from "./components/Sidebar";
import { MainContent } from "./components/MainContent";
import { ResizeHandle } from "./components/ResizeHandle";
import { ArticleWebView } from "./components/ArticleWebView";
import { AddFeedDialog } from "./components/AddFeedDialog";
import { SettingsDialog } from "./components/SettingsDialog";
import { useAppUpdater } from "./hooks/useAppUpdater";

function App() {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const loadFeeds = useFeedStore((s) => s.loadFeeds);
  const loadArticles = useFeedStore((s) => s.loadArticles);
  const loadSettings = useSettingsStore((s) => s.loadSettings);
  const selectedArticle = useArticleViewStore((s) => s.selectedArticle);

  useAppUpdater();

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
      {selectedArticle && (
        <>
          <ResizeHandle />
          <ArticleWebView />
        </>
      )}
      <AddFeedDialog open={dialogOpen} onClose={() => setDialogOpen(false)} />
      <SettingsDialog
        open={settingsOpen}
        onClose={() => setSettingsOpen(false)}
      />
    </div>
  );
}

export default App;
