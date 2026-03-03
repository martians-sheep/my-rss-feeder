import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useFeedStore } from "../stores/feedStore";

interface OgpResultData {
  ogImageUrl: string | null;
  ogImageLocal: string | null;
  ogDescription: string | null;
}

interface OgpResult {
  articleId: string;
  success: boolean;
  data: OgpResultData | null;
  error: string | null;
}

export function useOgpFetch() {
  const articles = useFeedStore((s) => s.articles);
  const updateArticleOgp = useFeedStore((s) => s.updateArticleOgp);
  const fetchingRef = useRef<Set<string>>(new Set());

  useEffect(() => {
    const unfetched = articles.filter(
      (a) => !a.ogFetched && !fetchingRef.current.has(a.id),
    );
    if (unfetched.length === 0) return;

    const ids = unfetched.map((a) => a.id);
    for (const id of ids) {
      fetchingRef.current.add(id);
    }

    invoke<OgpResult[]>("fetch_ogp_batch", { articleIds: ids })
      .then((results) => {
        for (const result of results) {
          if (result.success && result.data) {
            updateArticleOgp(result.articleId, {
              ogImageUrl: result.data.ogImageUrl,
              ogImageLocal: result.data.ogImageLocal,
              ogDescription: result.data.ogDescription,
            });
          } else {
            updateArticleOgp(result.articleId, {
              ogImageUrl: null,
              ogImageLocal: null,
              ogDescription: null,
            });
          }
        }
      })
      .catch(() => {
        // Mark all as fetched to avoid infinite retries
        for (const id of ids) {
          updateArticleOgp(id, {
            ogImageUrl: null,
            ogImageLocal: null,
            ogDescription: null,
          });
        }
      })
      .finally(() => {
        for (const id of ids) {
          fetchingRef.current.delete(id);
        }
      });
  }, [articles, updateArticleOgp]);
}
