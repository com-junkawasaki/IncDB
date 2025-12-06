import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

// GraphQL API プロキシ
export const POST: RequestHandler = async ({ request }) => {
	try {
		const body = await request.json();
		
		// 実際の GraphQL サーバーに転送
		// サーバーサイドでは環境変数から取得、クライアントサイドでは相対パスを使用
		const apiUrl = process.env.API_URL || import.meta.env.VITE_API_URL || (typeof window === 'undefined' ? 'http://api:8080' : 'http://localhost:8080');
		const graphqlUrl = `${apiUrl}/graphql`;
		
		const response = await fetch(graphqlUrl, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
			},
			body: JSON.stringify(body),
		});
		
		const data = await response.json();
		return json(data);
	} catch (error) {
		console.error('GraphQL request failed:', error);
		return json(
			{ errors: [{ message: 'GraphQL request failed' }] },
			{ status: 500 }
		);
	}
};

