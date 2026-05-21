module sentinel-orchestrator

go 1.21

require (
	google.golang.org/grpc v1.60.0
	google.golang.org/protobuf v1.32.0
	go.etcd.io/etcd/client/v3 v3.5.10
	github.com/segmentio/kafka-go v0.4.46
	github.com/lib/pq v1.10.9
)

require (
	go.uber.org/multierr v1.11.0
	go.uber.org/zap v1.26.0
)
