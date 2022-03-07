package cluster

import (
	"fmt"
	"sync"
	"time"
)

type NodeInfo struct {
	Name      string
	Ip        string
	Port      int
	CreatedAt int64
	Active    bool
}

func (m *NodeInfo) Address() string {
	return fmt.Sprintf("%s:%d", m.Ip, m.Port)
}

type Node struct {
	nodeMembers         []*NodeInfo
	NMMutex             sync.RWMutex
	currentNode         *NodeInfo
	heartbeatTime       time.Duration
	connectTimeout      time.Duration
	memberFailTimeout   time.Duration
	memberRemoveTimeout time.Duration
}

type MembersUpdate struct {
	Name        string
	NodeMembers []*NodeInfo
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
		nodeMembers:         []*NodeInfo{},
		currentNode:         currentNode,
		heartbeatTime:       2 * time.Second,
		connectTimeout:      3 * time.Second,
		memberFailTimeout:   8 * time.Second,
		memberRemoveTimeout: 24 * time.Second,
	}

	return node
}
