package main

import (
	"fmt"
	"log"
	"net/http"
	"os"

	"sentinel-api-gateway/pkg/graphql"

	"github.com/go-chi/chi"
)

func main() {
	// Setup logger
	log.SetFlags(log.LstdFlags | log.Lshortfile)

	// Load config
	fusionEndpoint := os.Getenv("FUSION_ENDPOINT")
	if fusionEndpoint == "" {
		fusionEndpoint = "localhost:50051"
	}

	orchestratorEndpoint := os.Getenv("ORCHESTRATOR_ENDPOINT")
	if orchestratorEndpoint == "" {
		orchestratorEndpoint = "localhost:50052"
	}

	// Initialize GraphQL schema
	schema, err := graphql.NewSchema(fusionEndpoint, orchestratorEndpoint)
	if err != nil {
		log.Fatalf("Failed to create GraphQL schema: %v", err)
	}

	// Setup router
	router := chi.NewRouter()

	// GraphQL endpoint
	router.Post("/graphql", graphql.GraphQLHandler(schema))
	router.Get("/graphql", graphql.GraphQLPlaygroundHandler())

	// Health check
	router.Get("/health", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"status":"healthy"}`))
	})

	// Start server
	port := ":4000"
	log.Printf("API Gateway listening on %s", port)
	if err := http.ListenAndServe(port, router); err != nil {
		log.Fatalf("Server error: %v", err)
	}
}
