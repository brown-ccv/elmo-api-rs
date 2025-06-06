#!/usr/bin/env bash

set -e
set -u
set -o pipefail

gcloud config set project ccv-oscar-utilization

# Delete the VPC connector first
gcloud compute networks vpc-access connectors delete elmo-connector \
  --region=us-east4 

# Delete the Cloud NAT
gcloud compute routers nats delete elmo-nat \
  --router=elmo-router \
  --region=us-east4 

# Delete the Cloud Router
gcloud compute routers delete elmo-router \
  --region=us-east4 

# Delete the subnet
gcloud compute networks subnets delete elmo-subnet \
  --region=us-east4 

# Delete the VPC network
gcloud compute networks delete elmo-vpc --quiet

# Delete the static IP address
gcloud compute addresses delete elmo-vpc-static-ip \
  --region=us-east4 
