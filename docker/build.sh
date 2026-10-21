#!/bin/bash
set -ex

ROOT_DIR=$(readlink -f $(dirname $0)/..)

REGISTRY=${REGISTRY:-registry.yiliu.pub/gene-platform}

docker build \
    -t ${REGISTRY}/web-app \
    --target web-app \
    -f ${ROOT_DIR}/docker/Dockerfile \
    --push \
    ${ROOT_DIR}

docker build \
    -t ${REGISTRY}/web-server \
    --target web-server \
    -f ${ROOT_DIR}/docker/Dockerfile \
    --push \
    ${ROOT_DIR}
