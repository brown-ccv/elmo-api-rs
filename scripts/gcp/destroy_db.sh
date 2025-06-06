#!/usr/bin/env bash

set -e
set -x
set -o pipefail

gcloud sql instances delete elmo
