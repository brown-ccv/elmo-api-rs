#!/usr/bin/env bash

# FIXME: This script probably has some errors and at the very least, has some unnecessary 
# features. For example, the `elmo-vpc-static-ip` is not really necessary, as we end up 
# using a public IP address assigned to the Cloud SQL instance, and then a private IP 
# address also attached to the Cloud SQL instance, but inside the `elmo-vpc` network.
# 
# Also, the NAT configuration is not really necessary, as we end up using a public IP 
# address assigned to the Cloud SQL instance, and then a private IP address also attached 
# to the Cloud SQL instance, but inside the `elmo-vpc` network.
# 
# Also, the VPC access connector is not really necessary, as we end up using a public IP 



set -e             # Exit immediately if a command exits with a non-zero status
set -u             # Treat unset variables as an error when substituting
set -o pipefail    # Prevents errors in a pipeline from being masked  


gcloud config set project ccv-oscar-utilization

# Reserve a static external IP address
gcloud compute addresses create elmo-vpc-static-ip --region=us-east4

gcloud compute addresses describe elmo-vpc-static-ip --region=us-east4




# Create a VPC network
gcloud compute networks create elmo-vpc --subnet-mode=auto

# Create a subnet
gcloud compute networks subnets create elmo-subnet \
  --network=elmo-vpc \
  --region=us-east4 \
  --range=10.0.0.0/28




# Create a Cloud Router
gcloud compute routers create elmo-router \
  --network=elmo-vpc \
  --region=us-east4

# Configure NAT with the static IP
gcloud compute routers nats create elmo-nat \
  --router=elmo-router \
  --region=us-east4 \
  --nat-external-ip-pool=elmo-vpc-static-ip \
  --nat-all-subnet-ip-ranges



# Enable the Serverless VPC Access API
gcloud services enable vpcaccess.googleapis.com

# Create the VPC access connector
gcloud compute networks vpc-access connectors create elmo-connector \
  --region=us-east4 \
  --subnet=elmo-subnet \
  --subnet-project=ccv-oscar-utilization \
  --min-instances=2 \
  --max-instances=10 \
  --machine-type=e2-standard-4
