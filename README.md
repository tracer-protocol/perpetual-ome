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

#### Docker
Run the container with env variables as follows. Note the port mapping to expose the OME on port 8989
`docker run -e "executioner_ip=http://localhost:3000/submit" -p 8989:8989 tracer-ome`
