# Tracer Order Matching Engine #

This repository contains the source code for the Tracer Order Matching Engine (OME).

To get up to speed with development:

    $ git clone git@github.com:tracer-protocol/perpetual-ome.git
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
