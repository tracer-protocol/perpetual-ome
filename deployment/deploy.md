# Deployment
The OME may be run inside Kubernetes or in its stand alone docker container. It does however require ENV variables which must be setup in one of the following two ways.

## K8s

Ensure you have a local env file setup to create the OME ENV. A sample env file is as follows
```
OME_EXECUTIONER_ADDRESS=<IP>
```
Create a configmap holding the env variables for at least the executioner address as follows
`kubectl create configmap ome-env --from-env-file=.env`

You can now utilise the `deploy.yaml` file and deploy to a K8s cluster using `kubectl apply -f deploy.yaml`

#### Externally exposing the OME
Once your deployment is running, you will need to expose the deployment if you wish to access it externally.

##### Option 1: Load Balancer
The first option for doing this is to simply use an external load balancer. To do this, run `kubectl expose deployment.apps/tracer-ome --type=LoadBalancer`.

##### Option 2: Ingress
The second option is to use an ingress. The following details how to do this on GCP. First create a NodePort service using `kubectl apply -f service.yaml`. Now, you will need to create a static IP in GCP. Note the name of the IP.

Next, create a managed GCP certificate using `kubectl apply -f cert.yaml`. Finally to expose your pods to the world, first update the local `ingress.yaml` file and replace the `kubernetes.io/ingress.global-static-ip-name: ome-ingress-ip` line. Place the name of the IP you used here instead of `ome-ingress-ip`. You will then have to run `kubectl apply -f ingress.yaml` to create an ingress. Simply point your DNS provider to this ingress IP and you should be good to go accessing the OME at that IP. For more, see this (Google Guide)[https://cloud.google.com/kubernetes-engine/docs/how-to/managed-certs]. Alternatively you may directly access the OME from that IP.

## Docker
Run the container with env variables as follows. Note the port mapping to expose the OME on port 8989
`docker run -e "executioner_ip=<INSERT_IP>" -p 8989:8989 tracer-ome`
