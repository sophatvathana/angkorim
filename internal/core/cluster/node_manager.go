package cluster

import "sync"

type NodeManager struct {
	sync.RWMutex
	nodes map[string]*Node
}
