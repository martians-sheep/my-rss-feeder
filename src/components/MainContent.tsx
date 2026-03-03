import { useFeedStore } from "../stores/feedStore";
import { useOgpFetch } from "../hooks/useOgpFetch";
import { Toolbar } from "./Toolbar";
import { ArticleList } from "./ArticleList";
import { EmptyState } from "./EmptyState";

export function MainContent() {
  const articles = useFeedStore((s) => s.articles);
  useOgpFetch();

  return (
    <main className="flex flex-1 flex-col overflow-hidden">
      <Toolbar />
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
