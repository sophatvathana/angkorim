package store

import (
	"angkorim/internal/model"
	"fmt"
	"time"

	"angkorim/pkg/log"

	"github.com/lithammer/shortuuid/v4"
	"github.com/spf13/viper"
	"gorm.io/driver/postgres"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
	"gorm.io/gorm/schema"
	"gorm.io/plugin/dbresolver"
)

var (
	db *gorm.DB
)

type StoreInterface interface {
}

var Store StoreInterface

type store struct {
}

const DRIVER_SQLITE = "sqlite"
const DRIVER_POSTGRES = "postgres"

// Setup : Connect to mysql database
func Setup() {
	var err error
	var logMode = logger.Error
	switch viper.Get("database.driver") {
	case DRIVER_SQLITE:
		path := viper.GetString("database.sqlite.path")
		db, err = gorm.Open(sqlite.Open(path))
		if err != nil {
			log.Fatal(fmt.Sprintf("Failed to connect sqlite %s", err.Error()))
		} else {
			log.Info("Successfully connect to sqlite3, path: %s.", path)
			// db.LogMode(true)
		}
	case DRIVER_POSTGRES:
		host := viper.GetString("database.postgres.host")
		user := viper.GetString("database.postgres.user")
		password := viper.GetString("database.postgres.password")
		name := viper.GetString("database.postgres.name")
		port := viper.GetString("database.postgres.port")
		// charset := viper.GetString("database.postgres.charset")
		sslmode := viper.GetString("database.postgres.ssl")

		dsn := fmt.Sprintf("host=%s port=%s user=%s password=%s dbname=%s sslmode=%s", host, port, user, password, name, sslmode)
		db, err = gorm.Open(postgres.Open(dsn), &gorm.Config{
			Logger: logger.Default.LogMode(logMode),
			NamingStrategy: schema.NamingStrategy{
				SingularTable: true,
			},
		})
		if err != nil {
			log.Fatal(fmt.Sprintf("Failed to connect postgres %s", err.Error()))
		} else {
			log.Info("Successfully connect to postgres, database: %s.", name)
			db.Use(
				dbresolver.Register(dbresolver.Config{ /* xxx */ }).
					SetConnMaxIdleTime(time.Hour).
					SetConnMaxLifetime(24 * time.Hour).
					SetMaxIdleConns(viper.GetInt("database.mysql.pool.min")).
					SetMaxOpenConns(viper.GetInt("database.mysql.pool.max")),
			)
		}
	default:
		log.Fatal("We do not support this kind of storage system yet!")
	}
	// db.SingularTable(true)
	if err = db.AutoMigrate(model.Models...); nil != err {
		log.Error("auto migrate tables failed")
	}

	if err != nil {
		panic(err)
	}
}

func Tx(txFunc func(tx *gorm.DB) error) (err error) {
	tx := db.Begin()
	if tx.Error != nil {
		return
	}

	defer func() {
		if r := recover(); r != nil {
			tx.Rollback()
			panic(r) // re-throw panic after Rollback
		} else if err != nil {
			tx.Rollback()
		} else {
			err = tx.Commit().Error
		}
	}()

	err = txFunc(tx)
	return err
}

func (s store) CreateP2P(from *model.User, to *model.User) error {
	err := Tx(func(tx *gorm.DB) error {
		u := shortuuid.New()
		room := &model.Room{
			Name: u,
		}
		err := s.CreateRoom(room)
		err = s.CreateUserRoom(&model.UserRoom{
			UserID: from.ID,
			RoomID: room.ID,
		})
		err = s.CreateUserRoom(&model.UserRoom{
			UserID: to.ID,
			RoomID: room.ID,
		})
		return err
	})
	return err
}

func (s store) CreateRoom(t *model.Room) error {
	err := db.Create(t).Error
	return err
}

func (s store) CreateUserRoom(t *model.UserRoom) error {
	err := db.Create(t).Error
	return err
}

func init() {
	Store = store{}
}
