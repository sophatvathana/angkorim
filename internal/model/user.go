package model

import "database/sql"

type User struct {
	BaseModel
	Username sql.NullString `gorm:"size:32;unique;" json:"username" form:"username"`
	Email    sql.NullString `gorm:"size:128;unique;" json:"email" form:"email"`
	Phone    string         `gorm:"size:16;" json:"phone" form:"phone"`
	Password string         `gorm:"size:512" json:"password" form:"password"`
}
