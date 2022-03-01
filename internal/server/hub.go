package server

import (
	"angkorim/pkg/protocol"
	"sync"

	"google.golang.org/protobuf/proto"
)

type Clients map[string]*Client
type Hub struct {
	clients Clients
	sLock   sync.RWMutex

	topics map[string]Clients
	tLock  sync.RWMutex
}

func NewHub() *Hub {
	return &Hub{
		clients: Clients{},
		sLock:   sync.RWMutex{},
		topics:  map[string]Clients{},
		tLock:   sync.RWMutex{},
	}
}

func (h *Hub) AddClient(c *Client) {
	h.sLock.Lock()
	h.clients[c.UserId] = c
	h.sLock.Unlock()
}

func (h *Hub) Subscribe(c *Client, topics ...string) {
	h.tLock.Lock()
	defer h.tLock.Unlock()

	for _, topic := range topics {
		if nil == h.topics[topic] {
			h.topics[topic] = Clients{}
		}
		c.AddTopic(topic)
		h.topics[topic][c.UserId] = c
	}

}

func (b *Hub) Unsubscribe(s *Client, topics ...string) {
	for _, topic := range topics {
		b.tLock.Lock()
		if nil == b.topics[topic] {
			b.tLock.Unlock()
			continue
		}
		delete(b.topics[topic], s.UserId)
		b.tLock.Unlock()
		s.RemoveTopic(topic)
	}
}

func (b *Hub) Destroy(s *Client) {
	s.Close()
	b.sLock.Lock()
	b.Unsubscribe(s, s.GetTopics()...)
	delete(b.clients, s.UserId)
	defer b.sLock.Unlock()
}

func (b *Hub) Broadcast(payload proto.Message, topics ...string) {
	for _, topic := range topics {
		if b.Subscribers(topic) < 1 {
			continue
		}
		b.tLock.RLock()
		for _, s := range b.topics[topic] {
			go (func(s *Client) {
				s.Send(protocol.Command_CMD_SEND_MSG, payload, nil)
			})(s)
		}
		b.tLock.RUnlock()
	}
}

func (b *Hub) Subscribers(topic string) int {
	b.tLock.RLock()
	defer b.tLock.RUnlock()
	return len(b.topics[topic])
}

func (b *Hub) GetTopics() []string {
	b.tLock.RLock()
	brokerTopics := b.topics
	b.tLock.RUnlock()

	topics := []string{}
	for topic := range brokerTopics {
		topics = append(topics, topic)
	}

	return topics
}
