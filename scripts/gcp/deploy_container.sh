#!/usr/bin/env bash

set -e
set -u
set -o pipefail

echo 'Building and pushing the image...'

# Build for x86 
docker buildx build --debug --platform linux/amd64 -t elmo-api:latest  .

docker tag elmo-api:latest us-east4-docker.pkg.dev/ccv-oscar-utilization/elmo-containers/elmo-api:latest

# Push the image
docker push us-east4-docker.pkg.dev/ccv-oscar-utilization/elmo-containers/elmo-api:latest


source ./secrets/db_secrets.sh

gcloud run deploy elmo-service \
  --image=us-east4-docker.pkg.dev/ccv-oscar-utilization/elmo-containers/elmo-api:latest \
  --platform=managed \
  --region=us-east4 \
  --vpc-connector=elmo-connector \
  --allow-unauthenticated \
  --set-cloudsql-instances=elmo \
  --set-env-vars=DB_HOST=${DB_HOST},DB_PASSWORD=${DB_PASSWORD},DB_USER=${DB_USER},DB_NAME=${DB_NAME} \
  --port=3000
