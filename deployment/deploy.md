# Deployment
The OME may be run inside Kubernetes or in its stand alone docker container. It does however require ENV variables which must be setup in one of the following two ways.

## K8s
Create a configmap holding any env variables
`kubectl create configmap ome-env --from-env-file=.env`

You can now utilise the `deploy.yaml` file and deploy to a K8s cluster using `kubectl apply -f deploy.yaml`

Expose the ome internally to the k8s cluster with `kubectl apply -f service.yaml`

The OME will now be exposed as a service within your kubernetes cluster
## Releasing a new version

Ensure that `deployment/setupRelease.sh` is executable with `chmod +x deployment/setupRelease.sh`

Use the `deployment/setupRelease.sh` to help you to perform the following:

- update the version number in `Cargo.toml` using `cargo-bump`
- build a new version of the docker image tagged with the new version
- push the new docker image to gcr
- update the image in the Kubernetes deploy config

Now you can begin a rolling deploy by running `kubectl apply -f deploy.yaml`

## Docker
Run the container locally via docker. Note the port mapping to expose the OME on port 8989
`docker run -p 8989:8989 tracer-ome`
