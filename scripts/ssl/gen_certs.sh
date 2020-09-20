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
  echo "\$CERT_DIR not set, defaulting to ${DIR}/certs."
  CERT_DIR="${DIR}/certs"
fi

pushd ${DIR}

mkdir -p ${CERT_DIR}

echo "=> Generating valid CA."
openssl genrsa -passout file:${GEN_SSL_PASS} -aes256 \
  -out ${CERT_DIR}/ca.key 4096
openssl req -passin file:${GEN_SSL_PASS} -new -x509 \
  -days ${CERT_LIFETIME} -key ${CERT_DIR}/ca.key -out ${CERT_DIR}/ca.crt \
  -subj "/C=US/ST=California/L=Mountain View/O=Personal/OU=Server/CN=Root CA"

echo "=> Generating valid Server Key/Cert."
openssl genrsa -passout file:${GEN_SSL_PASS} -aes256 -out ${CERT_DIR}/server.key 4096
openssl req -passin file:${GEN_SSL_PASS} -new -key ${CERT_DIR}/server.key \
  -out ${CERT_DIR}/server.csr \
  -subj "/C=US/ST=California/L=Mountain View/O=Environment Tracker Node/OU=Server/CN=localhost"
openssl x509 -req -passin file:${GEN_SSL_PASS} -days ${CERT_LIFETIME} -in ${CERT_DIR}/server.csr \
  -CA ${CERT_DIR}/ca.crt -CAkey ${CERT_DIR}/ca.key -set_serial 01 \
  -out ${CERT_DIR}/server.crt

# echo "=> Removing passphrase from the Server Key."
# openssl rsa -passin file:${GEN_SSL_PASS} -in ${CERT_DIR}/server.key -out ${CERT_DIR}/server.key

echo "=> Generating valid Client Key/Cert."
openssl genrsa -passout file:${GEN_SSL_PASS} -aes256 -out ${CERT_DIR}/client.key 4096
openssl req -passin file:${GEN_SSL_PASS} -new -key ${CERT_DIR}/client.key \
  -out ${CERT_DIR}/client.csr \
  -subj "/C=US/ST=California/L=Mountain View/O=Environment Tracker Node/OU=Client/CN=localhost"
openssl x509 -req -passin file:${GEN_SSL_PASS} -days ${CERT_LIFETIME} -in ${CERT_DIR}/client.csr \
  -CA ${CERT_DIR}/ca.crt -CAkey ${CERT_DIR}/ca.key -set_serial 01 \
  -out ${CERT_DIR}/client.crt

# echo "=> Removing passphrase from the Client Key."
# openssl rsa -passin file:${GEN_SSL_PASS} -in ${CERT_DIR}/client.key -out ${CERT_DIR}/client.key

popd
