package cluster

import (
	pb "angkorim/pkg/protocol"
	"sync"
	"time"
)

type CRDTKeyValueStore struct {
	Data     *pb.CRDTKeyValueStore
	mutex    sync.RWMutex
	dataSync chan map[string]*pb.Element
}

func NewCRDTKeyValueStore() *CRDTKeyValueStore {
	return &CRDTKeyValueStore{
		Data: &pb.CRDTKeyValueStore{
			Elements: make(map[string]*pb.Element),
		},
		dataSync: make(chan map[string]*pb.Element),
	}
}

func (kv *CRDTKeyValueStore) Put(key, value string) {
	kv.mutex.Lock()
	defer kv.mutex.Unlock()

	timestamp := time.Now().UnixNano()
	element := pb.Element{
		Value:     value,
		Timestamp: timestamp,
	}

	kv.Data.Elements[key] = &element
	kv.dataSync <- map[string]*pb.Element{
		key: &element,
	}
}

func (kv *CRDTKeyValueStore) Get(key string) (string, bool) {
	kv.mutex.RLock()
	defer kv.mutex.RUnlock()

	element, ok := kv.Data.Elements[key]
	if !ok {
		return "", false
	}

	return element.Value, true
}

func (kv *CRDTKeyValueStore) Delete(key string) {
	kv.mutex.Lock()
	defer kv.mutex.Unlock()

	delete(kv.Data.Elements, key)
}
