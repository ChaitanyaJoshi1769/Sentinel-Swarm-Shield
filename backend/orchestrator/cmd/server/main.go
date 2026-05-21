package main

import (
	"context"
	"fmt"
	"log"
	"net"
	"os"

	"sentinel-orchestrator/pkg/orchestrator"
	"sentinel-orchestrator/pkg/service"

	"google.golang.org/grpc"
)

func main() {
	// Setup logger
	log.SetFlags(log.LstdFlags | log.Lshortfile)

	// Load config
	kafkaBrokers := os.Getenv("KAFKA_BROKERS")
	if kafkaBrokers == "" {
		kafkaBrokers = "localhost:9092"
	}

	etcdEndpoint := os.Getenv("ETCD_ENDPOINTS")
	if etcdEndpoint == "" {
		etcdEndpoint = "localhost:2379"
	}

	// Initialize orchestrator
	ctx := context.Background()
	orch, err := orchestrator.New(ctx, kafkaBrokers, etcdEndpoint)
	if err != nil {
		log.Fatalf("Failed to initialize orchestrator: %v", err)
	}

	// Start gRPC server
	listener, err := net.Listen("tcp", ":50052")
	if err != nil {
		log.Fatalf("Failed to listen: %v", err)
	}

	grpcServer := grpc.NewServer()
	service.RegisterOrchestratorServer(grpcServer, orch)

	log.Printf("Orchestrator listening on :50052")
	if err := grpcServer.Serve(listener); err != nil {
		log.Fatalf("Failed to serve: %v", err)
	}
}
