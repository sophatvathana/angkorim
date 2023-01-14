package core

import (
	"angkorim/pkg/log"
	"angkorim/pkg/protocol"
	"sync"
	"time"

	"github.com/gobwas/ws"
	"github.com/gobwas/ws/wsutil"
	"github.com/google/uuid"
	"github.com/panjf2000/gnet/v2"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"
)

type Client struct {
	hub       *Hub
	ConType   int8
	WSMutex   sync.RWMutex
	lock      sync.RWMutex
	WS        gnet.Conn
	wsOpCode  ws.OpCode
	UserId    string
	DeviceId  string
	topics    map[string]bool
	messages  chan *proto.Message
	createdAt int64
	closed    bool
}

func NewClient(ws gnet.Conn, hub *Hub) *Client {
	return &Client{
		ConType:   1, //@TODO TCP
		WSMutex:   sync.RWMutex{},
		WS:        ws,
		UserId:    uuid.NewString(),
		DeviceId:  uuid.NewString(),
		messages:  make(chan *proto.Message),
		createdAt: time.Now().UnixNano(),
		topics:    map[string]bool{},
		lock:      sync.RWMutex{},
		hub:       hub,
	}
}

func (c *Client) Write(message []byte) error {
	c.WSMutex.Lock()
	defer c.WSMutex.Unlock()
	// _, err := c.WS.Write(message)
	err := wsutil.WriteServerMessage(c.WS, c.wsOpCode, message)
	// return err
	// return c.WS.WriteMessage(websocket.BinaryMessage, bytes)
	return err
}

func (c *Client) HandleMessage(bytes []byte) {
	var req = new(protocol.Request)
	err := proto.Unmarshal(bytes, req)
	if err != nil {
		log.Error("unmarshal error", err)
		return
	}
	log.Debug("Request", req)

	switch req.Cmd {
	case protocol.Command_CMD_SIGNIN:
		c.SignIn(req)
	case protocol.Command_CMD_SUBSCRIBE_TOPIC:
		c.Subscribe(req)
	case protocol.Command_CMD_SEND_MSG:
		c.SendSimpleBroadcast(req)
	}

}

func (c *Client) SendSimpleBroadcast(req *protocol.Request) {
	var reqPayload protocol.SimpleMessageRequest
	err := proto.Unmarshal(req.Data, &reqPayload)
	if err != nil {
		log.Error("Unmarshal req payload error %f", err)
		return
	}
	var res = &protocol.SimpleMessageResponse{}
	res.Message = reqPayload.Message
	c.hub.Broadcast(res, reqPayload.Topic)
}

func (c *Client) Subscribe(req *protocol.Request) {
	var reqPayload protocol.SubscribeRequest
	err := proto.Unmarshal(req.Data, &reqPayload)
	if err != nil {
		log.Error("Unmarshal req payload error %f", err)
		return
	}
	c.hub.Subscribe(c, reqPayload.Topic)
}

func (c *Client) SignIn(req *protocol.Request) {
	var reqPayload protocol.SignInRequest
	log.Info("signin")
	err := proto.Unmarshal(req.Data, &reqPayload)
	if err != nil {
		log.Error("Unmarshal req payload error %f", err)
		return
	}
	log.Info(reqPayload.PhoneNumber)
	var res = &protocol.SignInResponse{
		Token: "",
	}
	c.Send(protocol.Command_CMD_SIGNIN, res, err)
	c.UserId = reqPayload.PhoneNumber
	c.DeviceId = reqPayload.DeviceId
	addr := c.WS.RemoteAddr().String()
	print(addr)
	c.hub.AddClient(c)
}

func (c *Client) Send(cmd protocol.Command, message proto.Message, err error) {
	var output = protocol.Response{
		Cmd: cmd,
	}

	if err != nil {
		status, _ := status.FromError(err)
		output.Code = protocol.ResponseCode_REQUEST_ERROR
		output.Message = status.Message()
	}

	if message != nil {
		msgBytes, err := proto.Marshal(message)
		if err != nil {
			log.Error("", err)
			return
		}
		output.Data = msgBytes
	}

	outputBytes, err := proto.Marshal(&output)
	if err != nil {
		log.Error("", err)
		return
	}

	err = c.Write(outputBytes)
	if err != nil {
		log.Error("", err)
		c.Close()
		return
	}
}

func (c *Client) AddTopic(topic string) {
	c.lock.Lock()
	c.topics[topic] = true
	c.lock.Unlock()
}

func (c *Client) RemoveTopic(topic string) {
	c.lock.Lock()
	delete(c.topics, topic)
	c.lock.Unlock()
}

func (c *Client) GetTopics() []string {
	c.lock.RLock()
	subscriberTopics := c.topics
	c.lock.RUnlock()

	topics := []string{}
	for topic := range subscriberTopics {
		topics = append(topics, topic)
	}
	return topics
}

func (c *Client) Close() {
	c.lock.Lock()
	c.closed = true
	c.hub.Destroy(c)
	c.lock.Unlock()
	close(c.messages)
}
