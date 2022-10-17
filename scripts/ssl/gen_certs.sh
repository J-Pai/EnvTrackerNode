#!/bin/bash

# Adapted from the following link:
# https://github.com/grpc/grpc/issues/9593#issuecomment-277946137

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
CERT_LIFETIME=365

if [ ! -f ${DIR}/ssl_passphrase.txt ]; then
  echo -e "Please create the file ${DIR}/ssl_passphrase.txt.\n"
  echo -e "   This file should contain the passphrase for the SSL certificates."
  echo -e "   Please keep the passphrase a secret.\n"
  echo -e "   echo \"PASSPHRASE\" > ${DIR}/ssl_passphrase.txt"
  exit 1
fi

GEN_SSL_PASS=${DIR}/ssl_passphrase.txt

if [ -z "${CERT_DIR}" ]; then
  echo -e "\$CERT_DIR not set, defaulting to ${DIR}/certs.\n"
  CERT_DIR="${DIR}/certs"
fi

mkdir -p ${CERT_DIR}

function generate_root_ca() {
  echo "=> Generating valid CA."
  openssl genrsa -passout file:${GEN_SSL_PASS} -aes256 \
    -out ${CERT_DIR}/ca.key 4096
  openssl req -passin file:${GEN_SSL_PASS} -new -x509 \
    -days ${CERT_LIFETIME} -key ${CERT_DIR}/ca.key -out ${CERT_DIR}/ca.crt \
    -subj "/C=US/ST=California/L=Mountain View/O=Organization/OU=Root/CN=Root CA"
}

function generate_key_cert() {
  echo "=> Generating valid $1 Key/Cert."
  local domain=$2
  if [ -z "$2" ]; then
    echo "==> Domain not specified, defaulting to localhost."
    domain=localhost
  fi
  openssl genrsa -passout file:${GEN_SSL_PASS} -aes256 -out ${CERT_DIR}/$1.key 4096
  openssl req -passin file:${GEN_SSL_PASS} -new -key ${CERT_DIR}/$1.key \
    -out ${CERT_DIR}/$1.csr \
    -subj "/C=US/ST=California/L=Mountain View/O=Organization/OU=$1/CN=$domain"
  openssl x509 -req -passin file:${GEN_SSL_PASS} -days ${CERT_LIFETIME} -in ${CERT_DIR}/$1.csr \
    -CA ${CERT_DIR}/ca.crt -CAkey ${CERT_DIR}/ca.key -set_serial 01 \
    -out ${CERT_DIR}/$1.crt

  echo "=> Removing passphrase from the $1 Key."
  openssl rsa -passin file:${GEN_SSL_PASS} -in ${CERT_DIR}/$1.key -out ${CERT_DIR}/$1.key
}

if [ ! -f ${CERT_DIR}/ca.crt ] || [ ! -f ${CERT_DIR}/ca.key ]; then
  generate_root_ca
fi

echo -e "\nGenerate certificates using: generate_key_cert \$TARGET."
