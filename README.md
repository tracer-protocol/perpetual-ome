# Tracer Order Matching Engine #

This repository contains the source code for the Tracer Order Matching Engine (OME).

To get up to speed with development:

    $ git clone git@github.com:securedatalinks/tracer-ome.git
    $ cd tracer-ome
    $ rustup override set nightly
    $ cargo build
    $ cargo doc --open # read the manual
    $ grep TODO src/*.rs

## Setup guide
Follow the above steps to install all the required dependencies.

To set the debugging level, use
`export RUST_LOG=info`
To run the OME, with the executioner running locally, use
`cargo run -- --executioner_address "http://localhost:3000" --force-no-tls`

## ENV Variables
The OME supports the following ENV variables
- executioner_address: The IP address of the executioner instance
- port: The listening port of the OME
- address: The listening address of the OME
- dumpfile: The filepath to dump all orders on shutdown

## Deployment
To deploy changes to GCP, use the following.

### Build and tag the image
`docker build . -t gcr.io/tracer-protocol-testing/ome`

### Push to GCR
The executioner can easily be deployed to GCP by running the following.
`docker push gcr.io/tracer-protocol-testing/ome`

### Running
The OME may be run inside Kubernetes or in its stand alone docker container. It does however require ENV variables which must be setup in one of the following two ways.

#### K8s
Create a configmap holding the env variables for at least the executioner address as follows
`kubectl create configmap ome-env --from-env-file=.env`

You can now utilise the `deploy.yaml` file and deploy to a K8s cluster using `kubectl apply -f deploy.yaml`

Once your deployment is running, you will need to expose the deployment if you wish to access it externally.

The first option for doing this is to simply use an external load balancer. To do this, run `kubectl expose deployment.apps/tracer-ome --type=LoadBalancer`.

The second option is to use an ingress. The following details how to do this on GCP. First create a NodePort service using `kubectl apply -f service.yaml`. Now, you will need to create a static IP in GCP called `ome-ingress-ip`. Next, create a managed GCP certificate using `kubectl apply -f cert.yaml`. Finally to expose your pods to the world, you have to run `kubectl apply -f ingress.yaml` to create an ingress. Simply point your DNS provider to this ingress IP and you should be good to go accessing the OME at that IP. For more, see this (Google Guide)[https://cloud.google.com/kubernetes-engine/docs/how-to/managed-certs]

#### Docker
Run the container with env variables as follows. Note the port mapping to expose the OME on port 8989
`docker run -e "executioner_ip=<INSERT_IP>" -p 8989:8989 tracer-ome`
