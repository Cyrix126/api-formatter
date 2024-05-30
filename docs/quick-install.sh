#!/bin/sh
if grep -q Debian /etc/os-release
then
  # crate binary and config directories
  sudo mkdir -p /opt/api-formatter /etc/api-formatter
  # set right permission to directories
  sudo chown -R $USER /opt/api-formatter /etc/api-formatter
  # download binary
  wget https://github.com/Cyrix126/api-formatter/releases/download/v0.1.0/api-formatter-linux-amd64 /opt/api-formatter/
  # set execution permission for binary
  sudo chmod +x /opt/api-formatter/api-formatter-linux-amd64
  # download configuration
  wget https://raw.githubusercontent.com/Cyrix126/api-formatter/docs/config.toml /etc/api-formatter/config.toml
  # download systemd service
  wget https://raw.githubusercontent.com/Cyrix126/api-formatter/docs/api-formatter.service /etc/systemd/system/api-formatter.service
  # reload systemctl and enable service at boot
  sudo systemctl-daemon reload
  sudo systemctl enable api-formatter
  sudo systemctl stop api-formatter
  echo "Please customize the configuration file at /etc/api-formatter and start the program with\nsudo service api-formatter start"
else
  echo "This script is only for Debian distribution, abort." 
fi
