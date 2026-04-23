#!/usr/bin/env bash
export PYTHONPATH=$(pwd)/src
python3 -m ulf_orchestrator -c test_ulf.yml -i 50 --dry-run
