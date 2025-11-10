import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const GITHUB_TOKEN = import.meta.env.VITE_GITHUB_TOKEN;
const GITHUB_USERNAME = import.meta.env.VITE_GITHUB_USERNAME || 'qasimsk20';

export const GET: RequestHandler = async () => {
	if (!GITHUB_TOKEN) {
		return json({ error: 'GitHub token not configured' }, { status: 500 });
	}

	try {
		// Fetch user data
		const userResponse = await fetch(`https://api.github.com/users/${GITHUB_USERNAME}`, {
			headers: {
				Authorization: `Bearer ${GITHUB_TOKEN}`,
				Accept: 'application/vnd.github.v3+json'
			}
		});

		if (!userResponse.ok) {
			throw new Error(`GitHub API error: ${userResponse.status}`);
		}

		const userData = await userResponse.json();

		// Fetch repositories
		const reposResponse = await fetch(
			`https://api.github.com/users/${GITHUB_USERNAME}/repos?per_page=100`,
			{
				headers: {
					Authorization: `Bearer ${GITHUB_TOKEN}`,
					Accept: 'application/vnd.github.v3+json'
				}
			}
		);

		if (!reposResponse.ok) {
			throw new Error(`GitHub API error: ${reposResponse.status}`);
		}

		const repos = await reposResponse.json();

		// Calculate stats
		const stats = {
			repos: userData.public_repos,
			followers: userData.followers,
			following: userData.following,
			totalRepos: repos.length
		};

		return json(stats, {
			headers: {
				'Cache-Control': 'public, max-age=3600' // Cache for 1 hour
			}
		});
	} catch (error) {
		console.error('Error fetching GitHub stats:', error);
		return json(
			{ error: 'Failed to fetch GitHub stats' },
			{ status: 500 }
		);
	}
};
