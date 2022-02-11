package store

import (
	"angkorim/internal/model"
	"fmt"
	"time"

	"angkorim/internal/log"

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
