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
`cargo run -- --executioner_address "http://localhost:3000"`

## Common Issues


### The SSL certificate is invalid; class=Ssl (16); code=Certificate (-17)
Operating system - Ubuntu 18.04
Author - Dospore

#### Fix
export SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt


### `#![feature]` may not be used on the stable release channel
Operating system - Ubuntu 18.04
Author - Dospore

#### Explanation
Since we are using some experimental apis we need to tell rustc to use the unstable toolchain.
Some helpful links
- [Switching betwwen toolchains](https://stackoverflow.com/questions/58226545/how-to-switch-between-rust-toolchains)
- [Rust toolchains](https://doc.rust-lang.org/book/appendix-07-nightly-rust.html)

#### Fix
Create a rust-toolchain file in the root of your project with your desired toolchain as per [Switching betwwen toolchains](https://stackoverflow.com/questions/58226545/how-to-switch-between-rust-toolchains)

## Deployment
To deploy changes to GCP, use the following. Note tthat currently the executioner endpoint is defined in the Dockerfile. Ensure this endpoint is correctly set before building and pushing to GCP.
### Build and tag the image
`docker build . -t gcr.io/tracer-protocol-testing/ome`

### Push to GCR
The executioner can easily be deployed to GCP by running the following.
`docker push gcr.io/tracer-protocol-testing/ome`
