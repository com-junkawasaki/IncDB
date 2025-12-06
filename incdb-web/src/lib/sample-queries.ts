// Sample queries for IncDB

export interface SampleQuery {
	name: string;
	description: string;
	query: string;
	queryType: 'datalog' | 'graphql';
	category: 'data' | 'query' | 'sample';
}

export const sampleQueries: SampleQuery[] = [
	{
		name: 'Load Crypto Investigation Data',
		description: 'Create crypto crime investigation demo data (suspects, addresses, transactions)',
		query: `mutation {
  loadCryptoInvestigationData {
    createdCount
    createdIds
  }
}`,
		queryType: 'graphql',
		category: 'sample',
	},
	{
		name: 'Load Sample Data',
		description: 'Create sample Person, Company, and relationship data',
		query: `mutation {
  loadSampleData {
    createdCount
    createdIds
  }
}`,
		queryType: 'graphql',
		category: 'sample',
	},
	{
		name: 'Create Person',
		description: 'Create a new Person incidence',
		query: `mutation {
  addIncidence(
    level: 0
    value: { str: "Charlie" }
    embedding: [0.3, 0.4, 0.5, 0.6, 0.7]
  ) {
    id
    level
    value {
      str
    }
    embedding
  }
}`,
		queryType: 'graphql',
		category: 'data',
	},
	{
		name: 'Create Company',
		description: 'Create a new Company incidence',
		query: `mutation {
  addIncidence(
    level: 0
    value: { str: "Tech Inc" }
    embedding: [0.25, 0.35, 0.45, 0.55, 0.65]
  ) {
    id
    level
    value {
      str
    }
    embedding
  }
}`,
		queryType: 'graphql',
		category: 'data',
	},
	{
		name: 'Get All Incidences',
		description: 'Retrieve all incidences',
		query: `query {
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
    }
    embedding
  }
}`,
		queryType: 'graphql',
		category: 'query',
	},
	{
		name: 'Get Schema Stats',
		description: 'Get database statistics',
		query: `query {
  schemaStats {
    totalIncidences
    typeCount
    roleCount
    vectorCount
  }
}`,
		queryType: 'graphql',
		category: 'query',
	},
	{
		name: 'Get All Types',
		description: 'List all types in the database',
		query: `query {
  types {
    id
    level
    incidenceCount
  }
}`,
		queryType: 'graphql',
		category: 'query',
	},
	{
		name: 'Datalog: Find All Incidences',
		description: 'Find all incidences using Datalog',
		query: `Inc(1)
Inc(2)
Inc(3)`,
		queryType: 'datalog',
		category: 'query',
	},
	{
		name: 'Vector Search',
		description: 'Search for similar vectors',
		query: `query {
  vectorSearch(
    queryVector: [0.1, 0.2, 0.3, 0.4, 0.5]
    k: 5
  ) {
    incidences {
      incidence {
        id
        value {
          str
        }
      }
      similarity
    }
  }
}`,
		queryType: 'graphql',
		category: 'query',
	},
];

