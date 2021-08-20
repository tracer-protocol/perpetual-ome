#!/bin/sh
docker_image="gcr.io/tracer-protocol-testing/ome"
current_version="$(grep -oh -m 1 '[0-9].[0-9].[0-9]' Cargo.toml)"

cargo bump
version="$(grep -oh -m 1 '[0-9].[0-9].[0-9]' Cargo.toml)"

echo "updated version in Cargo.toml from $current_version to $version"

cargo update --package tracer-ome
echo "updated version in Cargo.lock from $current_version to $version"

read -p "create new docker image with tag $docker_image:$version? (y/n): " confirm_version

if [ $confirm_version != "y" ]
then
  echo "exiting"
  exit 1
fi

echo "running docker build . -t $docker_image:$version"
docker build . -t $docker_image:$version

echo "running docker push $docker_image:$version"
docker push $docker_image:$version

echo "built and pushed $docker_image:$version"

read -p "update k8s deployment config with new image tag? (y/n): " update_kubernetes

if [ $update_kubernetes != "y" ]
then
  echo "exiting"
  exit 1
fi

# update the deploy.yaml file
sed -i -e "s|$docker_image:[0-9].[0-9].[0-9]|$docker_image:$version|g" deployment/deploy.yaml

echo "updated k8s deployment config with new image"