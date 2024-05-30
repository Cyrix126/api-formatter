#!/bin/sh
# check that script is on supported distribution.
if grep -q debian /etc/os-release
then
  sudo apt install wget -y
  # crate binary and config directories
  sudo mkdir -p /opt/api-formatter /etc/api-formatter
  # set right permission to directories
  sudo chown -R $USER /opt/api-formatter /etc/api-formatter

  # download binary
  rm -f /opt/api-formatter/api-formatter-linux-amd64
  wget -q -P /opt/api-formatter/ https://github.com/Cyrix126/api-formatter/releases/download/v0.1.0/api-formatter-linux-amd64
  # set execution permission for binary
  sudo chmod +x /opt/api-formatter/api-formatter-linux-amd64

  # download configuration if it doesnt' exist.
  if [ ! -f /etc/api-formatter/config.toml ]
  then
  wget -q -P /etc/api-formatter/config.toml https://raw.githubusercontent.com/Cyrix126/api-formatter/main/docs/config.toml
  fi
  if [ ! -f /etc/api-formatter/config.toml ]
  then
  # download systemd service if it doesn't exist
 sudo wget -q -P /etc/systemd/system https://raw.githubusercontent.com/Cyrix126/api-formatter/main/docs/api-formatter.service   
 fi

# reload systemctl and enable service at boot
  sudo systemctl-daemon reload
  sudo systemctl enable api-formatter
  sudo systemctl stop api-formatter
  echo "Please customize the configuration file at /etc/api-formatter and start the program with\nsudo service api-formatter start"
else
  echo "This script is only for Debian distribution, abort." 
fi
