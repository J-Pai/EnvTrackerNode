#include "ssl_key_cert.h"

corenode::SslKeyCert::SslKeyCert() {
  const char* key_path = std::getenv("SSL_KEY");
  const char* cert_path = std::getenv("SSL_CERT");
  const char* root_path = std::getenv("SSL_ROOT_CERT");
  char resolved_path[PATH_MAX];

  if (!key_path) {
    throw std::runtime_error("$SSL_KEY not defined.");
  }
  realpath(key_path, resolved_path);
  ReadFile(resolved_path, key);

  if (!cert_path) {
    throw std::runtime_error("$SSL_CERT not defined.");
  }
  realpath(cert_path, resolved_path);
  ReadFile(resolved_path, cert);

  if (!root_path) {
    throw std::runtime_error("$SSL_ROOT_CERT not defined.");
  }
  realpath(root_path, resolved_path);
  ReadFile(resolved_path, root);
}

corenode::SslKeyCert::SslKeyCert(const std::string& key_path,
    const std::string& cert_path, const std::string& root_path) {
  ReadFile(key_path, key);
  ReadFile(cert_path, cert);
  ReadFile(root_path, root);
}

std::shared_ptr<grpc::ServerCredentials> corenode::SslKeyCert::GenerateServerCredentials() {
  grpc::SslServerCredentialsOptions::PemKeyCertPair key_cert = {
    key,
    cert
  };

  grpc::SslServerCredentialsOptions ssl_opts;
  ssl_opts.pem_root_certs = root;
  ssl_opts.pem_key_cert_pairs.push_back(key_cert);

  return grpc::SslServerCredentials(ssl_opts);
}

std::shared_ptr<grpc::ChannelCredentials> corenode::SslKeyCert::GenerateChannelCredentials() {
  grpc::SslCredentialsOptions ssl_opts = {
    root,
    key,
    cert
  };

  return grpc::SslCredentials(ssl_opts);
}

void corenode::SslKeyCert::ReadFile(const std::string& filename, std::string& data) {
  std::ifstream file(filename.c_str(), std::ios::in);
  if (file.is_open()) {
    std::stringstream string_stream;
    string_stream << file.rdbuf();
    file.close();
    data = string_stream.str();
  }
}

std::string corenode::SslKeyCert::getKey() {
  return std::string(key);
}

std::string corenode::SslKeyCert::getCert() {
  return std::string(cert);
}

std::string corenode::SslKeyCert::getRoot() {
  return std::string(root);
}
