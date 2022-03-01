package model

type Device struct {
	BaseModel
	UserID    int64
	UserAgent string `gorm:"size:255;" json:"status" form:"status"`
	Status    int    `gorm:"size:1;" json:"status" form:"status"` // Online status, 0: offline; 1: online
}
