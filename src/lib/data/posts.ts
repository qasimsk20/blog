export type CatppuccinColor =
  | "rosewater"
  | "flamingo"
  | "pink"
  | "mauve"
  | "red"
  | "maroon"
  | "peach"
  | "yellow"
  | "green"
  | "teal"
  | "sky"
  | "sapphire"
  | "blue"
  | "lavender";

export interface BlogTag {
  name: string;
  color: CatppuccinColor;
}

export interface BlogPost {
  slug: string;
  title: string;
  excerpt: string;
  body: string;
  publishedAt: string;
  tags: BlogTag[];
  readingTime: string;
}

export interface BlogPostSummary {
  slug: string;
  title: string;
  excerpt: string;
  publishedAt: string;
  publishedLabel: string;
  readingTime: string;
  tags: BlogTag[];
}

interface BlogPostSeed {
  slug: string;
  title: string;
  excerpt: string;
  body: string;
  publishedAt: string;
  tags: BlogTag[];
}

export const DEFAULT_LOCALE = "en-GB";

export const DEFAULT_DATE_FORMAT: Intl.DateTimeFormatOptions = {
  year: "numeric",
  month: "short",
  day: "2-digit",
};

const seeds: BlogPostSeed[] = [];

export const samplePosts: BlogPost[] = seeds.map((seed) => ({
  ...seed,
  readingTime: formatReadingTime(seed.body),
}));

export function formatReadingTime(
  content: string,
  wordsPerMinute = 200,
): string {
  const words = content.trim().split(/\s+/).filter(Boolean).length;
  const minutes = Math.max(1, Math.ceil(words / wordsPerMinute));
  return `${minutes} min read`;
}

export function formatPublishedDate(
  dateInput: string | Date,
  locale: string = DEFAULT_LOCALE,
  options: Intl.DateTimeFormatOptions = DEFAULT_DATE_FORMAT,
): string {
  const date = typeof dateInput === "string" ? new Date(dateInput) : dateInput;
  if (Number.isNaN(date.getTime())) {
    return typeof dateInput === "string" ? dateInput : dateInput.toString();
  }
  return new Intl.DateTimeFormat(locale, options).format(date);
}

export function getAllPosts(): BlogPost[] {
  return [...samplePosts].sort((a, b) =>
    a.publishedAt > b.publishedAt ? -1 : 1,
  );
}

export function getPostSummaries(
  locale: string = DEFAULT_LOCALE,
): BlogPostSummary[] {
  return getAllPosts().map((post) => toPostSummary(post, locale));
}

export function getRecentPostSummaries(
  limit: number = 3,
  locale: string = DEFAULT_LOCALE,
): BlogPostSummary[] {
  const count = Math.max(0, Math.floor(limit));
  return getPostSummaries(locale).slice(0, count);
}

export function getPostBySlug(slug: string): BlogPost | undefined {
  return samplePosts.find((post) => post.slug === slug);
}

export function searchPosts(query: string): BlogPost[] {
  const normalised = query.trim().toLowerCase();
  if (!normalised) {
    return getAllPosts();
  }

  return samplePosts.filter((post) => {
    const tagNames = post.tags.map((t) => t.name);
    const haystack = [post.title, post.excerpt, post.body, ...tagNames]
      .join(" ")
      .toLowerCase();

    return haystack.includes(normalised);
  });
}

export function toPostSummary(
  post: BlogPost,
  locale: string = DEFAULT_LOCALE,
): BlogPostSummary {
  return {
    slug: post.slug,
    title: post.title,
    excerpt: post.excerpt,
    publishedAt: post.publishedAt,
    publishedLabel: formatPublishedDate(post.publishedAt, locale).toLowerCase(),
    readingTime: post.readingTime,
    tags: [...post.tags],
  };
}
