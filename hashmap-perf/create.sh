#!/bin/bash
#openssl rand -out 2m $((2*1024*1024))
#openssl rand -out 50m $((50*1024*1024))

base64 /dev/urandom | head -c $((2*1024*1024)) > 2m
base64 /dev/urandom | head -c $((50*1024*1024)) > 50m
