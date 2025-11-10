declare module "markdown-it" {
	interface Options {
		html?: boolean;
		xhtmlOut?: boolean;
		breaks?: boolean;
		linkify?: boolean;
		typographer?: boolean;
		highlight?: (str: string, lang: string) => string | undefined;
	}

	interface Utils {
		escapeHtml(str: string): string;
	}

	export default class MarkdownIt {
		constructor(options?: Options);

		render(src: string, env?: unknown): string;
		utils: Utils;
	}
}
