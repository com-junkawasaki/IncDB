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
 * Convert GraphQL structure to SvelteFlow format
 */
export function convertToSvelteFlow(
	nodes: GraphQLGraphNode[],
	edges: GraphQLGraphEdge[],
): { nodes: any[]; edges: any[] } {
	const svelteFlowNodes: any[] = nodes.map((node) => {
		const label = node.value?.str || node.id;
		const level = node.level || 0;
		return {
			id: node.id,
			type: 'default',
			position: { x: Math.random() * 800, y: Math.random() * 600 },
			data: {
				label: label,
				level: level,
				typeId: node.typeId || 'N/A'
			},
			style: {
				background: getNodeColor(level),
				color: '#1d1d1f',
				border: `2px solid ${getNodeColor(level)}`,
				borderRadius: '8px',
				padding: '8px 12px',
				fontSize: '14px',
				fontWeight: 500
			}
		};
	});

	const svelteFlowEdges: any[] = edges.map((edge, index) => ({
		id: `e${edge.from}-${edge.to}-${edge.role}-${index}`,
		source: edge.from,
		target: edge.to,
		label: edge.role ? `R${edge.role}` : '',
		type: 'smoothstep',
		animated: false,
		style: {
			stroke: '#86868b',
			strokeWidth: 2
		},
		labelStyle: {
			fill: '#86868b',
			fontSize: '12px',
			fontWeight: 500
		}
	}));

	return { nodes: svelteFlowNodes, edges: svelteFlowEdges };
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

