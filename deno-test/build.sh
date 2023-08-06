#!/bin/bash

cargo build -r
docker build -t filipton/deno-proxy:latest .
docker image push filipton/deno-proxy:latest
