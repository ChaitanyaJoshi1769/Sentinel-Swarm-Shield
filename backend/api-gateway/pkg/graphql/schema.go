package graphql

import (
	"encoding/json"
	"fmt"
	"net/http"

	"github.com/graphql-go/graphql"
)

// NewSchema creates the GraphQL schema
func NewSchema(fusionEndpoint, orchestratorEndpoint string) (graphql.Schema, error) {
	// Track type
	trackType := graphql.NewObject(graphql.ObjectConfig{
		Name: "Track",
		Fields: graphql.Fields{
			"id": &graphql.Field{Type: graphql.String},
			"latitude": &graphql.Field{Type: graphql.Float},
			"longitude": &graphql.Field{Type: graphql.Float},
			"altitude": &graphql.Field{Type: graphql.Float},
			"confidence": &graphql.Field{Type: graphql.Float},
			"droneType": &graphql.Field{Type: graphql.String},
			"threatLevel": &graphql.Field{Type: graphql.String},
		},
	})

	// Threat type
	threatType := graphql.NewObject(graphql.ObjectConfig{
		Name: "Threat",
		Fields: graphql.Fields{
			"id": &graphql.Field{Type: graphql.String},
			"droneType": &graphql.Field{Type: graphql.String},
			"attackProbability": &graphql.Field{Type: graphql.Float},
			"timeToImpact": &graphql.Field{Type: graphql.Float},
			"priorityScore": &graphql.Field{Type: graphql.Int},
		},
	})

	// Root query
	queryType := graphql.NewObject(graphql.ObjectConfig{
		Name: "Query",
		Fields: graphql.Fields{
			"tracks": &graphql.Field{
				Type: graphql.NewList(trackType),
				Resolve: func(p graphql.ResolveParams) (interface{}, error) {
					// Mock implementation
					return []map[string]interface{}{}, nil
				},
			},
			"threats": &graphql.Field{
				Type: graphql.NewList(threatType),
				Resolve: func(p graphql.ResolveParams) (interface{}, error) {
					// Mock implementation
					return []map[string]interface{}{}, nil
				},
			},
			"status": &graphql.Field{
				Type: graphql.NewObject(graphql.ObjectConfig{
					Name: "Status",
					Fields: graphql.Fields{
						"healthy": &graphql.Field{Type: graphql.Boolean},
						"activeTracks": &graphql.Field{Type: graphql.Int},
						"activeThreats": &graphql.Field{Type: graphql.Int},
					},
				}),
				Resolve: func(p graphql.ResolveParams) (interface{}, error) {
					return map[string]interface{}{
						"healthy": true,
						"activeTracks": 0,
						"activeThreats": 0,
					}, nil
				},
			},
		},
	})

	schema, err := graphql.NewSchema(graphql.SchemaConfig{
		Query: queryType,
	})

	return schema, err
}

// GraphQLHandler handles GraphQL requests
func GraphQLHandler(schema graphql.Schema) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		var request map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&request); err != nil {
			http.Error(w, "Invalid request", http.StatusBadRequest)
			return
		}

		query := request["query"].(string)
		result := graphql.Do(graphql.Params{
			Schema:        schema,
			RequestString: query,
		})

		if result.HasErrors() {
			w.Header().Set("Content-Type", "application/json")
			w.WriteHeader(http.StatusBadRequest)
			json.NewEncoder(w).Encode(result.Errors)
			return
		}

		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(result)
	}
}

// GraphQLPlaygroundHandler serves GraphQL Playground
func GraphQLPlaygroundHandler() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/html")
		fmt.Fprint(w, `
<!DOCTYPE html>
<html>
<head>
	<title>GraphQL Playground</title>
	<style>
		body { margin: 0; }
		html { height: 100%; overflow: hidden; }
		#root { height: 100%; }
	</style>
	<script src="https://cdn.jsdelivr.net/npm/graphiql@1/graphiql.min.js"></script>
	<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/graphiql@1/graphiql.min.css" />
</head>
<body>
	<div id="root"></div>
	<script>
		ReactDOM.render(
			React.createElement(GraphiQL, {
				fetcher: GraphiQL.createFetcher({
					url: '/graphql',
				}),
			}),
			document.getElementById('root'),
		);
	</script>
</body>
</html>
		`)
	}
}
