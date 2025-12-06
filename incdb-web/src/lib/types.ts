// Type definitions for IncDB Web UI

export interface IId {
	id: string;
}

export interface Incidence {
	id: string;
	level: number;
	typeId?: string | null;
	args: string[];
	roles: number[];
	value?: Value | null;
	embedding?: number[] | null;
}

export interface Value {
	str?: string | null;
	int?: number | null;
	float?: number | null;
	bool?: boolean | null;
	vector?: number[] | null;
}

export interface TypeInfo {
	id: string;
	level: number;
	incidenceCount: number;
}

export interface TypeDetail {
	id: string;
	level: number;
	incidenceCount: number;
	incidences: Incidence[];
	structurePatterns: StructurePattern[];
}

export interface StructurePattern {
	pattern: string;
	count: number;
}

export interface SchemaStats {
	totalIncidences: number;
	typeCount: number;
	roleCount: number;
	vectorCount: number;
}

export interface DatalogResult {
	incidences: Incidence[];
	predicateCount: number;
}

export interface VectorSearchItem {
	incidence: Incidence;
	similarity: number;
}

export interface VectorSearchResult {
	incidences: VectorSearchItem[];
}

export interface GraphNode {
	id: string;
	level: number;
	typeId?: string | null;
	value?: Value | null;
}

export interface GraphEdge {
	from: string;
	to: string;
	role: number;
}

export interface GraphStructure {
	nodes: GraphNode[];
	edges: GraphEdge[];
}

