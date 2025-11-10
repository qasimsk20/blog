import type { PageLoad } from "./$types";
import type { BlogTag } from "$lib/data/posts";
import { getPostSummaries } from "$lib/data/posts";

export const load: PageLoad = () => {
  const posts = getPostSummaries();

  // Extract unique tags with their colors
  const tagMap = new Map<string, BlogTag>();
  posts.forEach((post) => {
    post.tags.forEach((tag) => {
      if (!tagMap.has(tag.name)) {
        tagMap.set(tag.name, tag);
      }
    });
  });

  const tags = Array.from(tagMap.values()).sort((a, b) =>
    a.name.localeCompare(b.name),
  );

  return {
    posts,
    tags,
    meta: {
      total: posts.length,
    },
  };
};
