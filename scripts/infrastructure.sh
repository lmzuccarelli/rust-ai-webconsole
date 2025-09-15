#!/bin/bash

USER="${REMOTE_USER:-lzuccarelli}"
PK="${PK_ID:?PK_ID environment variable must be set}"
MS="ai-webconsole"
DESCRIPTION="Simple htmx based webconsole written in Rust"
REPO="https://github.com/lmzuccarelli/rust-ai-webconsole.git"
REPO_NAME="rust-ai-webconsole"
CLEAN=$1

create_configs() {
tee config/${MS}-config.json <<EOF
{
  "name": "${MS}-service",
  "description": "${DESCRIPTION}",
  "port": "1337",
  "certs_dir": "/home/${USER}/certs",
  "cert_mode": "file",
  "log_level": "debug",
  "db_path": "/home/${USER}/database",
  "deploy_dir": "/home/${USER}/ai-workloads/staging",
  "static_dir": "/home/${USER}/services/static"
}
EOF

tee config/${MS}.service <<EOF
[Unit]
Description=${MS}-service

[Service]
ExecStart=/home/${USER}/services/${MS}-service --config /home/${USER}/services/${MS}-config.json
Restart=Always
PIDFile=/tmp/${MS}_service_pid
EOF
}

clone_build_service() {
  HOSTS=("george")
  for host in "${HOSTS[@]}"; do
    ssh -i "${PK}" "${USER}@${host}" -t "mkdir -p /home/${USER}/database && mkdir -p /home/${USER}/services && rm -rf /home/${USER}/services/${MS}-service"
    if [ "${CLEAN}" == "true" ];
    then
      ssh -i "${PK}" "${USER}@${host}" -t "mkdir -p /home/${USER}/Projects && rm -rf /home/${USER}/Projects/${REPO_NAME} && cd /home/${USER}/Projects && git clone ${REPO} && cd ${REPO_NAME} && make build"
    else 
      ssh -i "${PK}" "${USER}@${host}" -t "cd /home/lzuccarelli/Projects/${REPO_NAME} && git pull origin main --rebase && make build"
    fi
  done
}

deploy_service() {
  HOSTS=("george")
  for host in "${HOSTS[@]}"; do
    scp -i "${PK}" config/* "${USER}@${host}:/home/${USER}/services"
    scp -i "${PK}" -r static/ "${USER}@${host}:/home/${USER}/services"
    ssh -i "${PK}" "${USER}@${host}" -t "cp /home/${USER}/Projects/${REPO_NAME}/target/release/${REPO_NAME} /home/${USER}/services/${MS}-service"
    ssh -i "${PK}" "${USER}@${host}" -t "sudo cp /home/${USER}/services/${MS}.service /etc/systemd/system/"
  done
}


start_service() {
  ssh -i "${PK}" "${USER}@george" -t "sudo systemctl daemon-reload && sudo systemctl start ${MS}.service"
}

restart_service() {
  ssh -i "${PK}" "${USER}@george" -t "sudo systemctl daemon-reload && sudo systemctl restart ${MS}.service"
}

stop_service() {
  ssh -i "${PK}" "${USER}@george" -t "sudo systemctl daemon-reload && sudo systemctl stop ${MS}.service"
}

"$@"
