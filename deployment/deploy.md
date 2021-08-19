# Deployment
The OME may be run inside Kubernetes or in its stand alone docker container. It does however require ENV variables which must be setup in one of the following two ways.

## K8s
Create a configmap holding any env variables
`kubectl create configmap ome-env --from-env-file=.env`

You can now utilise the `deploy.yaml` file and deploy to a K8s cluster using `kubectl apply -f deploy.yaml`

Expose the ome internally to the k8s cluster with `kubectl apply -f service.yaml`

The OME will now be exposed as a service within your kubernetes cluster
## Docker
Run the container locally via docker. Note the port mapping to expose the OME on port 8989
`docker run -p 8989:8989 tracer-ome`
