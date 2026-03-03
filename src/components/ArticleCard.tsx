import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import type { Article } from "../types/feed";
import { useFeedStore } from "../stores/feedStore";

interface ArticleCardProps {
  article: Article;
}

function getImageSrc(article: Article): string | null {
  if (article.ogImageLocal) {
    return convertFileSrc(article.ogImageLocal);
  }
  if (article.ogImageUrl) {
    return article.ogImageUrl;
  }
  return null;
}

export function ArticleCard({ article }: ArticleCardProps) {
  const markArticleRead = useFeedStore((s) => s.markArticleRead);

  const handleClick = async () => {
    if (!article.isRead) {
      markArticleRead(article.id);
    }
    if (article.url) {
      await open(article.url);
    }
  };

  const imageSrc = getImageSrc(article);
  const description = article.ogDescription ?? article.summary;

  return (
    <article
      onClick={handleClick}
      className="group relative cursor-pointer rounded-lg border border-gray-200 bg-white transition-shadow hover:shadow-md"
    >
      <div className="flex">
        <div className="min-w-0 flex-1 p-4">
          <div className="flex gap-3">
            {!article.isRead && (
              <span
                data-testid="unread-indicator"
                className="mt-2 h-2.5 w-2.5 shrink-0 rounded-full bg-blue-500"
              />
            )}
            <div className="min-w-0 flex-1">
              <h3
                className={`text-sm font-semibold leading-snug ${article.isRead ? "text-gray-500" : "text-gray-900"}`}
              >
                {article.title}
              </h3>
              {description && (
                <p className="mt-1 line-clamp-2 text-xs text-gray-500">
                  {description}
                </p>
              )}
              <div className="mt-2 flex items-center gap-2 text-xs text-gray-400">
                {article.feedTitle && <span>{article.feedTitle}</span>}
                {article.publishedAt && (
                  <time dateTime={article.publishedAt}>
                    {new Date(article.publishedAt).toLocaleDateString()}
                  </time>
                )}
              </div>
            </div>
          </div>
        </div>
        <div className="flex w-24 shrink-0 items-center justify-center overflow-hidden rounded-r-lg">
          {imageSrc ? (
            <img
              src={imageSrc}
              alt={article.title}
              className="h-full w-full object-cover"
            />
          ) : (
            <div
              data-testid="og-placeholder"
              className="flex h-full w-full items-center justify-center bg-gray-100"
            >
              <svg
                className="h-8 w-8 text-gray-300"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={1.5}
                  d="M2.25 15.75l5.159-5.159a2.25 2.25 0 013.182 0l5.159 5.159m-1.5-1.5l1.409-1.409a2.25 2.25 0 013.182 0l2.909 2.909M3.75 21h16.5A2.25 2.25 0 0022.5 18.75V5.25A2.25 2.25 0 0020.25 3H3.75A2.25 2.25 0 001.5 5.25v13.5A2.25 2.25 0 003.75 21z"
                />
              </svg>
            </div>
          )}
        </div>
      </div>
    </article>
  );
}
