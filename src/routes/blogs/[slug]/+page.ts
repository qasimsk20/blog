import { error } from "@sveltejs/kit";
import type { PageLoad } from "./$types";
import {
  DEFAULT_LOCALE,
  formatPublishedDate,
  getAllPosts,
  getPostBySlug,
  toPostSummary,
} from "$lib/data/posts";

export const load: PageLoad = ({ params, url }) => {
  const { slug } = params;
  const localeParam = url.searchParams.get("locale");
  const locale =
    localeParam && localeParam.trim() ? localeParam.trim() : DEFAULT_LOCALE;

  const post = slug ? getPostBySlug(slug) : null;

  if (!post) {
    throw error(404, {
      message: `The post "${slug ?? "unknown"}" could not be found.`,
    });
  }

  const allPosts = getAllPosts();
  const currentIndex = allPosts.findIndex((entry) => entry.slug === post.slug);

  const previous =
    currentIndex > 0 ? toPostSummary(allPosts[currentIndex - 1], locale) : null;
  const next =
    currentIndex >= 0 && currentIndex < allPosts.length - 1
      ? toPostSummary(allPosts[currentIndex + 1], locale)
      : null;

  const related = allPosts
    .filter(
      (entry) =>
        entry.slug !== post.slug &&
        entry.tags.some((tag) => post.tags.some((pt) => pt.name === tag.name)),
    )
    .slice(0, 4)
    .map((entry) => toPostSummary(entry, locale));

  return {
    post: {
      ...post,
      publishedLabel: formatPublishedDate(
        post.publishedAt,
        locale,
      ).toLowerCase(),
    },
    adjacent: {
      previous,
      next,
    },
    related,
    meta: {
      locale,
    },
  };
};
