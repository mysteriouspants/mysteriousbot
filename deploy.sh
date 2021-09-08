#!/bin/sh

set -o xtrace

SERVICE_SU_USER=xpm
SERVICE_HOST=mysteriouspants.com
SERVICE_USER_NAME=mysteriousbot
SERVICE_USER_DIR=/home/mysteriousbot
SERVICE_NAME=mysteriousbot

docker run --rm --name mysteriousbot \
  -u $(id -u):$(id -g) \
  -v $(pwd):/src \
  -w /src \
  rust:1.54 \
  cargo build --release

# deploy the build if successful
if [ $? -eq 0 ]; then
  ssh ${SERVICE_SU_USER}@${SERVICE_HOST} sudo systemctl stop ${SERVICE_NAME}
  ssh ${SERVICE_USER_NAME}@${SERVICE_HOST} mkdir -p ${SERVICE_USER_DIR}/config
  ssh ${SERVICE_USER_NAME}@${SERVICE_HOST} mkdir -p ${SERVICE_USER_DIR}/db
  rsync -avzr config/mysteriousbot.toml ${SERVICE_USER_NAME}@${SERVICE_HOST}:${SERVICE_USER_DIR}/config/mysteriousbot.toml
  rsync -avzr target/release/mysteriousbot ${SERVICE_USER_NAME}@${SERVICE_HOST}:${SERVICE_USER_DIR}/mysteriousbot
  ssh ${SERVICE_USER_NAME}@${SERVICE_HOST} chmod +x ${SERVICE_USER_DIR}/mysteriousbot
  ssh ${SERVICE_SU_USER}@${SERVICE_HOST} sudo systemctl start ${SERVICE_NAME}
else
  echo "Fix your broken build, man."
fi
