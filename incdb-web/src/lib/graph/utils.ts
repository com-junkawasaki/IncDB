// Graph processing utilities

import type { GraphQLGraphNode, GraphQLGraphEdge } from '$lib/graphql/types';

export interface GraphNode {
	id: string;
	label: string;
	level: number;
	color: string;
}

export interface GraphEdge {
	from: string;
	to: string;
	label: string;
	arrows: string;
}

/**
 * Convert GraphQL structure to vis-network format
 */
export function convertToVisNetwork(
	nodes: GraphQLGraphNode[],
	edges: GraphQLGraphEdge[],
): { nodes: GraphNode[]; edges: GraphEdge[] } {
	const visNodes: GraphNode[] = nodes.map((node) => ({
		id: node.id,
		label: node.id,
		level: node.level,
		color: getNodeColor(node.level),
	}));

	const visEdges: GraphEdge[] = edges.map((edge) => ({
		from: edge.from,
		to: edge.to,
		label: `R${edge.role}`,
		arrows: 'to',
	}));

	return { nodes: visNodes, edges: visEdges };
}

/**
 * Get node color based on level
 */
export function getNodeColor(level: number): string {
	const colors = ['#007aff', '#34c759', '#ff9500', '#ff3b30', '#af52de'];
	return colors[level % colors.length] || '#86868b';
}

/**
 * Filter graph structure by node IDs
 */
export function filterGraph(
	nodes: GraphQLGraphNode[],
	edges: GraphQLGraphEdge[],
	nodeIds: string[],
): { nodes: GraphQLGraphNode[]; edges: GraphQLGraphEdge[] } {
	const nodeSet = new Set(nodeIds);
	const filteredNodes = nodes.filter((node) => nodeSet.has(node.id));
	const filteredEdges = edges.filter(
		(edge) => nodeSet.has(edge.from) && nodeSet.has(edge.to),
	);
	return { nodes: filteredNodes, edges: filteredEdges };
}

