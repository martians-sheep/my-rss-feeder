import { open } from "@tauri-apps/plugin-shell";
import { useFeedStore } from "../stores/feedStore";
import { useArticleViewStore } from "../stores/articleViewStore";
import { useOgpFetch } from "../hooks/useOgpFetch";
import { Toolbar } from "./Toolbar";
import { ArticleList } from "./ArticleList";
import { EmptyState } from "./EmptyState";

export function MainContent() {
  const articles = useFeedStore((s) => s.articles);
  const selectedArticle = useArticleViewStore((s) => s.selectedArticle);
  const articleListWidth = useArticleViewStore((s) => s.articleListWidth);
  const closeWebview = useArticleViewStore((s) => s.closeWebview);
  useOgpFetch();

  const isWebviewOpen = selectedArticle !== null;

  const handleOpenExternal = async () => {
    if (selectedArticle?.url) {
      await open(selectedArticle.url);
    }
  };

  return (
    <main
      className={
        isWebviewOpen
          ? "flex shrink-0 flex-col overflow-hidden border-r border-gray-200"
          : "flex flex-1 flex-col overflow-hidden"
      }
      style={isWebviewOpen ? { width: `${articleListWidth}px` } : undefined}
    >
      <Toolbar />
      {isWebviewOpen && (
        <div className="flex items-center gap-1 border-b border-gray-200 px-3 py-1.5">
          <p className="min-w-0 flex-1 truncate text-xs text-gray-600">
            {selectedArticle.title}
          </p>
          <button
            onClick={handleOpenExternal}
            className="shrink-0 rounded p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-600"
            title="外部ブラウザで開く"
          >
            <svg
              className="h-3.5 w-3.5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M13.5 6H5.25A2.25 2.25 0 003 8.25v10.5A2.25 2.25 0 005.25 21h10.5A2.25 2.25 0 0018 18.75V10.5m-10.5 6L21 3m0 0h-5.25M21 3v5.25"
              />
            </svg>
          </button>
          <button
            onClick={closeWebview}
            className="shrink-0 rounded p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-600"
            title="閉じる"
          >
            <svg
              className="h-3.5 w-3.5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>
      )}
      <div className="flex-1 overflow-y-auto p-6">
        {articles.length > 0 ? (
          <ArticleList articles={articles} />
        ) : (
          <EmptyState />
        )}
      </div>
    </main>
  );
}
