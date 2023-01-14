package cluster

import (
	"angkorim/pkg/protocol"
	"fmt"
	"log"
	"math/rand"
	"net"
	"os"
	"sync"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/reflection"
)

type NodeInfo struct {
	Name      string `json:"name"`
	Ip        string `json:"ip"`
	Port      int    `json:"port"`
	CreatedAt int64
	Active    bool
}

func (m *NodeInfo) Address() string {
	return fmt.Sprintf("%s:%d", m.Ip, m.Port)
}

type Node struct {
	NMMutex             sync.RWMutex
	currentNode         *NodeInfo
	heartbeatTime       time.Duration
	connectTimeout      time.Duration
	memberFailTimeout   time.Duration
	memberRemoveTimeout time.Duration
	status              protocol.NodeStatus
	updatedAt           int64
}

func NewNode(name string, host string, port int) *Node {
	currentNode := &NodeInfo{
		Name:      name,
		Ip:        host,
		Port:      port,
		CreatedAt: time.Now().UnixNano(),
		Active:    true,
	}

	node := &Node{
		currentNode:         currentNode,
		heartbeatTime:       2 * time.Second,
		connectTimeout:      3 * time.Second,
		memberFailTimeout:   8 * time.Second,
		memberRemoveTimeout: 24 * time.Second,
		status:              protocol.NodeStatus_ALIVE_NODE,
		updatedAt:           time.Now().UnixNano(),
	}

	return node
}

func (n *Node) ListenTCP() {
	rand.Seed(time.Now().UTC().UnixNano())
	listener, err := net.Listen("tcp", n.currentNode.Address())

	if err != nil {
		log.Fatalf("failed to listen: %v", err)
		os.Exit(1)
	} else {
		log.Println("Listening and serving Node TCP on", listener.Addr().String())
	}
	grpcServer := grpc.NewServer()
	reflection.Register(grpcServer)
	protocol.RegisterClusterServer(grpcServer, NodeRPCSever{})
	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		if err := grpcServer.Serve(listener); err != nil {
			log.Fatalf("failed to serve: %v", err)
		}
		wg.Done()
	}()
	//TODO Join Node
	wg.Wait()
}
