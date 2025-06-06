#!/usr/bin/env bash

set -euo pipefail

# Create an Artifact Registry repository
gcloud artifacts repositories create elmo-containers --repository-format=docker --location=us-east4

# Configure Docker to use gcloud as a credential helper
gcloud auth configure-docker us-east4-docker.pkg.dev

