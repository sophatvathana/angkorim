package cluster

import (
	"angkorim/pkg/protocol"
	"context"
)

type NodeRPCSever struct {
}

func (ns NodeRPCSever) SeyHello(context.Context, *protocol.RequestHello) (*protocol.ResponseHello, error) {
	return nil, nil
}
