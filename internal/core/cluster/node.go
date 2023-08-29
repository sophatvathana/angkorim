package cluster

import (
	pb "angkorim/pkg/protocol"
	"context"
	"fmt"
	"log"
	"net"
	"sync"
	"time"

	"github.com/gofrs/uuid"
	"google.golang.org/grpc"
)

type NodeInfo struct {
	Id            string
	Name          string `json:"name"`
	Ip            string `json:"ip"`
	Port          int    `json:"port"`
	CreatedAt     int64
	Active        bool
	Members       []NodeInfo
	Status        pb.NodeStatus
	Address       string
	KeyValueStore *CRDTKeyValueStore
	LastHeardFrom int64
}

func (m *NodeInfo) GetAddress() string {
	return fmt.Sprintf("%s:%d", m.Ip, m.Port)
}

func NewNode(name string, host string, port int, members []NodeInfo) *ClusterServer {
	nodeID, _ := uuid.NewV4()
	CurrentNode := &NodeInfo{
		Id:            nodeID.String(),
		Name:          name,
		Ip:            host,
		Port:          port,
		CreatedAt:     time.Now().UnixNano(),
		Active:        true,
		Members:       members,
		Status:        pb.NodeStatus_ALIVE_NODE,
		KeyValueStore: NewCRDTKeyValueStore(),
	}

	node := &ClusterServer{
		CurrentNode: CurrentNode,
		nodes:       make(map[string]*NodeInfo),
	}

	return node
}

type ClusterServer struct {
	pb.UnimplementedMembershipServer
	CurrentNode *NodeInfo
	nodes       map[string]*NodeInfo
	nodesLock   sync.RWMutex
}

func (s *ClusterServer) Ping(ctx context.Context, req *pb.PingRequest) (*pb.PingResponse, error) {
	s.nodesLock.RLock()
	defer s.nodesLock.RUnlock()

	if existingMember, ok := s.nodes[req.Sender.Id]; ok {
		existingMember.Address = req.Sender.Address
	} else {
		s.nodes[req.Sender.Id] = &NodeInfo{
			Id:      req.Sender.Id,
			Address: req.Sender.Address,
		}
	}

	members := make([]*pb.Member, 0, len(s.nodes))
	for _, m := range s.nodes {
		members = append(members, &pb.Member{
			Id:      m.Id,
			Address: m.Address,
		})
	}

	return &pb.PingResponse{
		Ack:     true,
		Members: members,
		PongFrom: &pb.Member{
			Id:      s.CurrentNode.Id,
			Address: s.CurrentNode.Address,
		},
	}, nil
}

func (s *ClusterServer) Join(ctx context.Context, req *pb.JoinRequest) (*pb.JoinResponse, error) {
	s.nodesLock.Lock()
	defer s.nodesLock.Unlock()
	log.Printf("Request join from node %s", req.NewMember.Address)
	if _, ok := s.nodes[req.NewMember.Id]; !ok {
		s.nodes[req.NewMember.Id] = &NodeInfo{
			Id:      req.NewMember.Id,
			Address: req.NewMember.Address,
		}
	}

	members := make([]*pb.Member, 0, len(s.nodes))
	for _, m := range s.nodes {
		members = append(members, &pb.Member{
			Id:      m.Id,
			Address: m.Address,
		})
	}

	return &pb.JoinResponse{
		Ack:     true,
		Members: members,
		NodeToJoin: &pb.Member{
			Id:      s.CurrentNode.Id,
			Address: s.CurrentNode.Address,
		},
	}, nil
}

func (s *ClusterServer) pushCRDT(node *NodeInfo, kvStore map[string]*pb.Element) {
	fmt.Printf("Push CRDT to %s \n", node.Address)
	conn, err := grpc.Dial(node.Address, grpc.WithInsecure())
	if err != nil {
		log.Printf("failed to connect to node %s: %v", node.Id, err)
		return
	}
	defer conn.Close()

	client := pb.NewMembershipClient(conn)

	stream, err := client.PushCRDT(context.Background())
	if err != nil {
		log.Printf("failed to push CRDT to node %s: %v", node.Id, err)
		return
	}
	stream.Send(&pb.PushCRDTRequest{
		Sender: &pb.Member{
			Id:      s.CurrentNode.Id,
			Address: s.CurrentNode.Address,
		},
		KvStore: &pb.CRDTKeyValueStore{
			Elements: kvStore,
		},
	})
	log.Printf("Pushed CRDT to node %s", node.Id)
	// s.CurrentNode.KeyValueStore.dataSync <- kvStore
}

func (s *ClusterServer) PushCRDT(stream pb.Membership_PushCRDTServer) error {
	s.nodesLock.Lock()
	defer s.nodesLock.Unlock()
	for {
		rev, err := stream.Recv()
		if err != nil {
			log.Printf("Error received CRDT %s \n", err.Error())
			return err
		}
		_, ok := s.nodes[rev.Sender.Id]
		if !ok {
			log.Printf("received CRDT push from unknown node %s", rev.Sender.Id)
			return nil
		}
		for key, value := range rev.KvStore.Elements {
			s.CurrentNode.KeyValueStore.Put(key, value.Value)
		}

		log.Printf("Received CRDT push from node %s", rev.Sender.Id)

	}
}

func (s *ClusterServer) requestJoinBack(address, nodeID string) {
	if existingMember, ok := s.nodes[nodeID]; ok {
		existingMember.Address = address
		existingMember.LastHeardFrom = time.Now().Unix()
	} else {
		s.nodes[nodeID] = &NodeInfo{
			Id:            nodeID,
			Address:       address,
			LastHeardFrom: time.Now().Unix(),
		}
	}

}

func (s *ClusterServer) changeNodeStatus(nodeID string, status pb.NodeStatus) {
	if existingMember, ok := s.nodes[nodeID]; ok {
		existingMember.Status = status
	}
}

func (s *ClusterServer) startNode(address, nodeID string) {
	conn, err := grpc.Dial(address, grpc.WithInsecure())
	if err != nil {
		log.Fatalf("failed to connect: %v", err)
	}
	defer conn.Close()

	client := pb.NewMembershipClient(conn)

	joinRepsonse, err := client.Join(context.Background(), &pb.JoinRequest{
		NewMember: &pb.Member{
			Id:      nodeID,
			Address: s.CurrentNode.Address,
		},
	})
	if err != nil {
		log.Println("failed to join: %v", err)
		return
	}
	s.requestJoinBack(joinRepsonse.NodeToJoin.Address, joinRepsonse.NodeToJoin.Id)

	log.Println("Joined the membership group")

	for {
		pingResponse, err := client.Ping(context.Background(), &pb.PingRequest{
			Sender: &pb.Member{
				Id:      nodeID,
				Address: s.CurrentNode.Address,
			},
		})
		if err != nil {
			log.Printf("failed to ping need to suspect: %v \n", err)
			s.changeNodeStatus(joinRepsonse.NodeToJoin.Id, pb.NodeStatus_SUSPECTED_NODE)
		}

		log.Println("Received ping response with members:", pingResponse.Members)
		time.Sleep(5 * time.Second)
	}
}

func (n *ClusterServer) ListenTCP(ks *ClusterServer) {
	n.CurrentNode.Address = n.CurrentNode.GetAddress()
	lis, err := net.Listen("tcp", n.CurrentNode.GetAddress())
	if err != nil {
		log.Fatalf("failed to listen: %v", err)
	}

	s := grpc.NewServer()
	pb.RegisterMembershipServer(s, ks)

	fmt.Printf("Server listening on port %d", n.CurrentNode.Port)
	go func() {
		if err := s.Serve(lis); err != nil {
			log.Fatalf("failed to serve: %v", err)
		}
	}()

	n.CurrentNode.KeyValueStore.dataSync = make(chan map[string]*pb.Element)
	go func() {
		for {
			select {
			case kvStore := <-n.CurrentNode.KeyValueStore.dataSync:
				for _, node := range n.nodes {
					fmt.Println(node.Id != n.CurrentNode.Id)
					if node.Id != n.CurrentNode.Id {
						n.pushCRDT(node, kvStore)
					}
				}
			}
		}
	}()

	fmt.Printf("Memebers to join %d \n", len(n.CurrentNode.Members))
	if len(n.CurrentNode.Members) > 0 {
		for _, member := range n.CurrentNode.Members {
			go func(member NodeInfo) {
				log.Printf("To join %s \n", member.GetAddress())
				n.startNode(member.GetAddress(), n.CurrentNode.Id)
			}(member)
		}
	}

	if err != nil {
		log.Fatalf("Failed to join: %v", err)
	}
}
