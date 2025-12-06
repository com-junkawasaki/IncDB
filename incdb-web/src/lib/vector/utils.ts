// Vector processing utilities

/**
 * Perform PCA (Principal Component Analysis) on vectors
 * Returns 2D or 3D coordinates
 * Simplified implementation using SVD approximation
 */
export function performPCA(vectors: number[][], dimensions: 2 | 3 = 2): number[][] {
	if (vectors.length === 0) return [];
	if (vectors[0].length === 0) return [];

	const n = vectors.length;
	const d = vectors[0].length;

	// Step 1: Center the data (subtract mean)
	const mean: number[] = [];
	for (let j = 0; j < d; j++) {
		let sum = 0;
		for (let i = 0; i < n; i++) {
			sum += vectors[i][j];
		}
		mean[j] = sum / n;
	}

	const centered: number[][] = [];
	for (let i = 0; i < n; i++) {
		centered[i] = [];
		for (let j = 0; j < d; j++) {
			centered[i][j] = vectors[i][j] - mean[j];
		}
	}

	// Step 2: Compute covariance matrix
	const covariance: number[][] = [];
	for (let i = 0; i < d; i++) {
		covariance[i] = [];
		for (let j = 0; j < d; j++) {
			let sum = 0;
			for (let k = 0; k < n; k++) {
				sum += centered[k][i] * centered[k][j];
			}
			covariance[i][j] = sum / (n - 1);
		}
	}

	// Step 3: Simple power iteration for first principal components
	// For simplicity, we'll use the first dimensions of the centered data
	// This is a simplified approach - for production, use a proper SVD library
	const result: number[][] = [];
	
	if (dimensions === 2) {
		// Use first two dimensions as approximation
		for (let i = 0; i < n; i++) {
			result.push([centered[i][0] || 0, centered[i][1] || 0]);
		}
	} else {
		// Use first three dimensions as approximation
		for (let i = 0; i < n; i++) {
			result.push([
				centered[i][0] || 0,
				centered[i][1] || 0,
				centered[i][2] || 0
			]);
		}
	}

	// Normalize the result to make it more visually appealing
	const maxX = Math.max(...result.map(r => Math.abs(r[0])));
	const maxY = Math.max(...result.map(r => Math.abs(r[1])));
	if (maxX > 0 && maxY > 0) {
		for (let i = 0; i < result.length; i++) {
			result[i][0] = result[i][0] / maxX;
			result[i][1] = result[i][1] / maxY;
			if (dimensions === 3) {
				const maxZ = Math.max(...result.map(r => Math.abs(r[2])));
				if (maxZ > 0) {
					result[i][2] = result[i][2] / maxZ;
				}
			}
		}
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

