#!/bin/bash

rust_in_podman() {
  podman run --rm --name mysteriousbot \
    -v $(pwd):/src:Z \
    -w /src \
    docker.io/library/rust:1.76.0 \
    $@
}

fast=false

while getopts :f arg
do
  case "${arg}" in
    f)
      fast=true
      ;;
    ?)
      echo "Invalid option -$OPTARG"
      exit 2
      ;;
  esac
done

set -o xtrace
set -x
set -e

SERVICE_SU_USER=xpm
SERVICE_HOST=mysteriouspants.com
SERVICE_USER_NAME=mysteriousbot
SERVICE_USER_DIR=/home/mysteriousbot
SERVICE_NAME=mysteriousbot

if [[ $fast == false ]]; then
  rust_in_podman cargo test -- --nocapture
  rust_in_podman cargo build --release
fi

# deploy the build if successful
if [ $? -eq 0 ]; then
  if [[ $fast == false ]]; then
    ssh ${SERVICE_SU_USER}@${SERVICE_HOST} sudo systemctl stop ${SERVICE_NAME}
    ssh ${SERVICE_USER_NAME}@${SERVICE_HOST} mkdir -p ${SERVICE_USER_DIR}/config
    ssh ${SERVICE_USER_NAME}@${SERVICE_HOST} mkdir -p ${SERVICE_USER_DIR}/db
  fi

  # pull down the production db as a backup - yay sqlite
  rsync -avzr ${SERVICE_USER_NAME}@${SERVICE_HOST}:${SERVICE_USER_DIR}/db db/db_$(date +%s)

  rsync -avzr config/mysteriousbot.yml ${SERVICE_USER_NAME}@${SERVICE_HOST}:${SERVICE_USER_DIR}/config/mysteriousbot.yml

  if [[ $fast == false ]]; then
    rsync -avzr target/release/mysteriousbot ${SERVICE_USER_NAME}@${SERVICE_HOST}:${SERVICE_USER_DIR}/mysteriousbot
    ssh ${SERVICE_USER_NAME}@${SERVICE_HOST} chmod +x ${SERVICE_USER_DIR}/mysteriousbot
    ssh ${SERVICE_SU_USER}@${SERVICE_HOST} sudo systemctl start ${SERVICE_NAME}
  else
    ssh ${SERVICE_SU_USER}@${SERVICE_HOST} sudo systemctl restart ${SERVICE_NAME}
  fi
else
  echo "Fix your broken build, man."
fi
