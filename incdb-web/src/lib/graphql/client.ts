// GraphQL client wrapper

import { GraphQLClient } from 'graphql-request';
import type {
	GraphQLIncidence,
	GraphQLTypeInfo,
	GraphQLTypeDetail,
	GraphQLSchemaStats,
	GraphQLDatalogResult,
	GraphQLVectorSearchResult,
	GraphQLGraphStructure,
} from './types';

const API_URL = import.meta.env.PUBLIC_API_URL || 'http://localhost:8080/graphql';

// For server-side rendering, use the API proxy
const isServer = typeof window === 'undefined';
const GRAPHQL_URL = isServer ? '/api/graphql' : API_URL;

const client = new GraphQLClient(GRAPHQL_URL);

export async function query<T>(query: string, variables?: Record<string, unknown>): Promise<T> {
	try {
		return await client.request<T>(query, variables);
	} catch (error) {
		console.error('GraphQL query error:', error);
		throw error;
	}
}

export async function getIncidences(): Promise<GraphQLIncidence[]> {
	const queryStr = `
		query {
			incidences {
				id
				level
				typeId
				args
				roles
				value {
					str
					int
					float
					bool
					vector
				}
				embedding
			}
		}
	`;
	const result = await query<{ incidences: GraphQLIncidence[] }>(queryStr);
	return result.incidences;
}

export async function getIncidence(id: string): Promise<GraphQLIncidence | null> {
	const queryStr = `
		query($id: String!) {
			incidence(id: $id) {
				id
				level
				typeId
				args
				roles
				value {
					str
					int
					float
					bool
					vector
				}
				embedding
			}
		}
	`;
	const result = await query<{ incidence: GraphQLIncidence | null }>(queryStr, { id });
	return result.incidence;
}

export async function getTypes(): Promise<GraphQLTypeInfo[]> {
	const queryStr = `
		query {
			types {
				id
				level
				incidenceCount
			}
		}
	`;
	const result = await query<{ types: GraphQLTypeInfo[] }>(queryStr);
	return result.types;
}

export async function getType(id: string): Promise<GraphQLTypeDetail | null> {
	const queryStr = `
		query($id: String!) {
			type(id: $id) {
				id
				level
				incidenceCount
				incidences {
					id
					level
					typeId
					args
					roles
					value {
						str
						int
						float
						bool
						vector
					}
					embedding
				}
				structurePatterns {
					pattern
					count
				}
			}
		}
	`;
	const result = await query<{ type: GraphQLTypeDetail | null }>(queryStr, { id });
	return result.type;
}

export async function getSchemaStats(): Promise<GraphQLSchemaStats> {
	const queryStr = `
		query {
			schemaStats {
				totalIncidences
				typeCount
				roleCount
				vectorCount
			}
		}
	`;
	const result = await query<{ schemaStats: GraphQLSchemaStats }>(queryStr);
	return result.schemaStats;
}

export async function executeDatalog(queryStr: string): Promise<GraphQLDatalogResult> {
	const mutation = `
		query($query: String!) {
			executeDatalog(query: $query) {
				incidences {
					id
					level
					typeId
					args
					roles
					value {
						str
						int
						float
						bool
						vector
					}
					embedding
				}
				predicateCount
			}
		}
	`;
	const result = await query<{ executeDatalog: GraphQLDatalogResult }>(mutation, { query: queryStr });
	return result.executeDatalog;
}

export async function vectorSearch(queryVector: number[], k: number): Promise<GraphQLVectorSearchResult> {
	const queryStr = `
		query($queryVector: [Float!]!, $k: Int!) {
			vectorSearch(queryVector: $queryVector, k: $k) {
				incidences {
					incidence {
						id
						level
						typeId
						args
						roles
						value {
							str
							int
							float
							bool
							vector
						}
						embedding
					}
					similarity
				}
			}
		}
	`;
	const result = await query<{ vectorSearch: GraphQLVectorSearchResult }>(queryStr, {
		queryVector,
		k,
	});
	return result.vectorSearch;
}

export async function getGraphStructure(ids: string[], depth: number): Promise<GraphQLGraphStructure> {
	const queryStr = `
		query($ids: [String!]!, $depth: Int!) {
			graphStructure(ids: $ids, depth: $depth) {
				nodes {
					id
					level
					typeId
					value {
						str
						int
						float
						bool
						vector
					}
				}
				edges {
					from
					to
					role
				}
			}
		}
	`;
	const result = await query<{ graphStructure: GraphQLGraphStructure }>(queryStr, { ids, depth });
	return result.graphStructure;
}

