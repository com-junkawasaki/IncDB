// GraphQL type definitions
// These types match the GraphQL schema

export interface GraphQLIncidence {
	id: string;
	level: number;
	typeId?: string | null;
	args: string[];
	roles: number[];
	value?: GraphQLValue | null;
	embedding?: number[] | null;
}

export interface GraphQLValue {
	str?: string | null;
	int?: number | null;
	float?: number | null;
	bool?: boolean | null;
	vector?: number[] | null;
}

export interface GraphQLTypeInfo {
	id: string;
	level: number;
	incidenceCount: number;
}

export interface GraphQLTypeDetail {
	id: string;
	level: number;
	incidenceCount: number;
	incidences: GraphQLIncidence[];
	structurePatterns: GraphQLStructurePattern[];
}

export interface GraphQLStructurePattern {
	pattern: string;
	count: number;
}

export interface GraphQLSchemaStats {
	totalIncidences: number;
	typeCount: number;
	roleCount: number;
	vectorCount: number;
}

export interface GraphQLDatalogResult {
	incidences: GraphQLIncidence[];
	predicateCount: number;
}

export interface GraphQLVectorSearchItem {
	incidence: GraphQLIncidence;
	similarity: number;
}

export interface GraphQLVectorSearchResult {
	incidences: GraphQLVectorSearchItem[];
}

export interface GraphQLGraphNode {
	id: string;
	level: number;
	typeId?: string | null;
	value?: GraphQLValue | null;
}

export interface GraphQLGraphEdge {
	from: string;
	to: string;
	role: number;
}

export interface GraphQLGraphStructure {
	nodes: GraphQLGraphNode[];
	edges: GraphQLGraphEdge[];
}

