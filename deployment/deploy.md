# Deployment
The OME may be run inside Kubernetes or in its stand alone docker container. It does however require ENV variables which must be setup in one of the following two ways.

## K8s
Create a configmap holding the env variables for at least the executioner address as follows
`kubectl create configmap ome-env --from-env-file=.env`

You can now utilise the `deploy.yaml` file and deploy to a K8s cluster using `kubectl apply -f deploy.yaml`

Once your deployment is running, you will need to expose the deployment if you wish to access it externally.

The first option for doing this is to simply use an external load balancer. To do this, run `kubectl expose deployment.apps/tracer-ome --type=LoadBalancer`.

The second option is to use an ingress. The following details how to do this on GCP. First create a NodePort service using `kubectl apply -f service.yaml`. Now, you will need to create a static IP in GCP called `ome-ingress-ip`. Next, create a managed GCP certificate using `kubectl apply -f cert.yaml`. Finally to expose your pods to the world, you have to run `kubectl apply -f ingress.yaml` to create an ingress. Simply point your DNS provider to this ingress IP and you should be good to go accessing the OME at that IP. For more, see this (Google Guide)[https://cloud.google.com/kubernetes-engine/docs/how-to/managed-certs]

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
Run the container with env variables as follows. Note the port mapping to expose the OME on port 8989
`docker run -e "executioner_ip=<INSERT_IP>" -p 8989:8989 tracer-ome`
