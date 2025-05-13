# Makefile for MongoDB ODBC Hybrid API Example

CC = gcc
CFLAGS = -Wall -g -I../../include
LDFLAGS = -L../../target/release -lmongo_odbc_core -lodbc -ldl -lpthread -lm

# Optionally link against OpenSSL if needed
# LDFLAGS += -lssl -lcrypto

all: hybrid_mongo_connector

hybrid_mongo_connector: hybrid_mongo_connector.c
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

clean:
	rm -f hybrid_mongo_connector

.PHONY: all clean
