#!/bin/bash
ulimit -n 50000

k6 run perf.js
