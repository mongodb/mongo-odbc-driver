# Makefile for all MongoDB ODBC Driver Examples

all: advanced_odbc_helper direct_api hybrid_api

advanced_odbc_helper:
	$(MAKE) -C advanced_odbc_helper

direct_api:
	$(MAKE) -C direct_api

hybrid_api:
	$(MAKE) -C hybrid_api

clean:
	$(MAKE) -C advanced_odbc_helper clean
	$(MAKE) -C direct_api clean
	$(MAKE) -C hybrid_api clean

.PHONY: all advanced_odbc_helper direct_api hybrid_api clean
