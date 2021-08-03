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
`cargo run -- --known_markets_url "http://localhost:3030/book" --external_book_url "http://localhost:3030/book/" --force-no-tls`

## ENV Variables
The OME supports the following ENV variables
- `known_markets_url`: an endpoint which will return a list of known markets upon a GET request
- `external_book_url`: an endpoint which will return an external book upon a GET request. marketId will be concatenated to this url
- `port`: The listening port of the OME
- `address`: The listening address of the OME

## Deployment
To deploy changes to GCP, use the following.

### Build and tag the image
`docker build . -t gcr.io/tracer-protocol-testing/ome`

### Push to GCR
The executioner can easily be deployed to GCP by running the following.
`docker push gcr.io/tracer-protocol-testing/ome`