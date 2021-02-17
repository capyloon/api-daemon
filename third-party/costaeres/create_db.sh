#!/bin/bash

set -x -e

rm build.sqlite
sqlite3 build.sqlite < db/migrations/00001_main.sql

