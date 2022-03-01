package model

import (
	"time"

	"gorm.io/gorm"
)

type UserRoom struct {
	UserID    string `sql:"type:uuid;"`
	RoomID    string `sql:"type:uuid;"`
	CreatedAt time.Time
	UpdatedAt time.Time
	DeletedAt gorm.DeletedAt `gorm:"index"`
}
