package model

import (
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"
)

var Models = []interface{}{
	&User{}, &Room{},
}

type BaseModel struct {
	ID        string `sql:"type:uuid;primary_key;default:uuid_generate_v4()"`
	CreatedAt time.Time
	UpdatedAt time.Time
	DeletedAt gorm.DeletedAt `gorm:"index"`
}

func (base *BaseModel) BeforeCreate(tx *gorm.DB) error {
	id, err := uuid.NewUUID()
	if err != nil {
		return err
	}
	return tx.Set("ID", id.String()).Error
}
