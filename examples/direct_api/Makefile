# Makefile for MongoDB ODBC Direct API Example

CC = gcc
CFLAGS = -Wall -g -I../../include
LDFLAGS = -L../../target/release -lmongo_odbc_core -ldl -lpthread -lm

# Optionally link against OpenSSL if needed
# LDFLAGS += -lssl -lcrypto

all: direct_mongo_connector

direct_mongo_connector: direct_mongo_connector.c
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

clean:
	rm -f direct_mongo_connector

.PHONY: all clean
