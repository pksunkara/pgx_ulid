#!/usr/bin/env bash
echo "shared_preload_libraries = 'ulid'" >> $PG_DATA/postgresql.conf
