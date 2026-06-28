#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd $SCRIPT_DIR

echo ">> Creating Users."

useradd -r -s /bin/false -U envtrackernode_node
usermod -aG envtrackernode_node envtrackernode_node

echo ">> Installing Files"
rm -rf /opt/envtrackernode_node/release
mkdir -p /opt/envtrackernode_node/release
cp -r $SCRIPT_DIR/node /opt/envtrackernode_node/release/node

if [ -f $SCRIPT_DIR/config.toml ]; then
	cp -f $SCRIPT_DIR/config.toml /opt/envtrackernode_node/config.toml
fi

echo ">> Creating service file."

cat << 'EOF' >| /etc/systemd/system/envtrackernode_node.service
[Unit]
Description=EnvTrackerNode -- node
After=network.target network-online.target bluetooth.service tailscaled.service multi-user.target
Wants=network.target network-online.target bluetooth.service tailscaled.service multi-user.target

[Service]
Type=exec
WorkingDirectory=/opt/envtrackernode_node/release
User=envtrackernode_node
Group=envtrackernode_node
Restart=always
RestartSec=5s

ExecStart=/opt/envtrackernode_node/release/node --config /opt/envtrackernode_node/config.toml

[Install]
WantedBy=multi-user.target
EOF

echo ">> Starting service."

systemctl daemon-reload
systemctl enable envtrackernode_node.service
systemctl restart envtrackernode_node.service

echo ">> Make sure to create a config file at:"
echo ">> /opt/envtrackernode_node/config.toml"
