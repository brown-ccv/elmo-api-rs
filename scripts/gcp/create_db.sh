#!/usr/bin/env bash

set -e
set -x
set -o pipefail

INSTANCE_NAME=elmo
VERSION=POSTGRES_17
TIER=db-g1-small
REGION=us-east4
SIZE=11     # units are GB

source ~/projects/elmo-api-rs/secrets/db_secrets.sh  # defines ROOT_PASSWORD

gcloud config set project ccv-oscar-utilization

gcloud sql instances create ${INSTANCE_NAME} \
  --database-version=${VERSION} \
  --tier=${TIER} \
  --region=${REGION}\
  --root-password=${ROOT_PASSWORD} \
  --availability-type=REGIONAL \
  --storage-size=${SIZE} \
  --storage-type=SSD \
  --backup-start-time=23:00 \
  --maintenance-window-day=SUN \
  --maintenance-window-hour=06 \
  --edition=enterprise \
  --network=projects/ccv-oscar-utilization/global/networks/elmo-vpc


echo "Sleeping for 60 seconds to allow the instance to be created"
sleep 60

# Add authorized network, otherwise we need to use the Cloud SQL proxy (see below)
gcloud sql instances patch elmo \
  --authorized-networks=128.148.194.11/32,128.148.128.57/32


# NOTE: The following is NOT NEEDED when we specify the authorized network above
#
# To connect to the database from an unauthorized network, use the Cloud SQL proxy:
# mkdir -p ~/software
# cd ~/software
# curl -o cloud-sql-proxy https://storage.googleapis.com/cloud-sql-connectors/cloud-sql-proxy/v2.15.2/cloud-sql-proxy.darwin.arm64
# chmod +x cloud-sql-proxy
# ./cloud-sql-proxy -instances=ccv-oscar-utilization:us-east4:elmo
