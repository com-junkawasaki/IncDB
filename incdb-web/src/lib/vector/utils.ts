// Vector processing utilities

import { Matrix } from 'ml-matrix';

/**
 * Perform PCA (Principal Component Analysis) on vectors
 * Returns 2D or 3D coordinates
 */
export function performPCA(vectors: number[][], dimensions: 2 | 3 = 2): number[][] {
	if (vectors.length === 0) return [];

	const matrix = new Matrix(vectors);
	const mean = matrix.mean('column');
	const centered = matrix.subRowVector(mean);
	const covariance = centered.transpose().mmul(centered).div(vectors.length);
	const { eigenvectors } = covariance.eigen();
	const principalComponents = eigenvectors.slice([0, dimensions - 1], [0, eigenvectors.columns - 1]);
	const projected = centered.mmul(principalComponents);

	const result: number[][] = [];
	for (let i = 0; i < projected.rows; i++) {
		const row: number[] = [];
		for (let j = 0; j < dimensions; j++) {
			row.push(projected.get(i, j));
		}
		result.push(row);
	}

	return result;
}

/**
 * Calculate cosine similarity between two vectors
 */
export function cosineSimilarity(a: number[], b: number[]): number {
	if (a.length !== b.length) return 0;
	const dot = a.reduce((sum, val, i) => sum + val * b[i], 0);
	const normA = Math.sqrt(a.reduce((sum, val) => sum + val * val, 0));
	const normB = Math.sqrt(b.reduce((sum, val) => sum + val * val, 0));
	if (normA === 0 || normB === 0) return 0;
	return dot / (normA * normB);
}

/**
 * Calculate similarity matrix for vectors
 */
export function calculateSimilarityMatrix(vectors: number[][]): number[][] {
	const n = vectors.length;
	const matrix: number[][] = [];

	for (let i = 0; i < n; i++) {
		const row: number[] = [];
		for (let j = 0; j < n; j++) {
			row.push(cosineSimilarity(vectors[i], vectors[j]));
		}
		matrix.push(row);
	}

	return matrix;
}

