#!/bin/sh
#
# Build Docker container and push it to Google Container Registry
#
# Environment needs $PROJECT_ID set to GCE's project.
#
#   ./scripts/deploy.sh [<docker tag>]
#
# If tag is not given, "latest" is used.
#

if [ ! -f "${PWD}/scripts/deploy.sh" ] && [ ! -f "${PWD}/Dockerfile" ]; then
    echo "ERROR: Run from project root: ./scripts/deploy.sh"
    exit 1
fi


# Exit on any error
set -e

if [ "${1}" ]; then
    TAG="${1}"
else
    TAG="latest"
fi

# No unbound variables allowed after this point
set -u

NAME=$(basename "${PWD}")

REMOTE_TAG="eu.gcr.io/${PROJECT_ID}/${NAME}:${TAG}"

docker build -t "${REMOTE_TAG}" .
gcloud docker -- push "${REMOTE_TAG}"
