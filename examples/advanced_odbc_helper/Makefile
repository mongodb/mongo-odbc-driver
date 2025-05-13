# Makefile for MongoDB Advanced ODBC Helper

CC = gcc
CFLAGS = -Wall -g -I/usr/include -I/usr/local/include
LDFLAGS = -lodbc

PROGRAMS = metadata_explorer error_analyzer connection_info_utility \
           data_type_handler mongodb_x509_connector

all: $(PROGRAMS)

metadata_explorer: metadata_explorer.c common.c
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

error_analyzer: error_analyzer.c common.c
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

connection_info_utility: connection_info_utility.c common.c
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

data_type_handler: data_type_handler.c common.c
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

mongodb_x509_connector: mongodb_x509_connector.c common.c
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

clean:
	rm -f $(PROGRAMS) *.o
