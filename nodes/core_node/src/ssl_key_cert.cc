#include "ssl_key_cert.h"

corenode::SslKeyCert::SslKeyCert() {
  const char* key_path = std::getenv("SSL_KEY");
  const char* cert_path = std::getenv("SSL_CERT");
  const char* root_path = std::getenv("SSL_ROOT_CERT");
  const char* json_path = std::getenv("CLIENT_SECRET_JSON");
  char resolved_path[PATH_MAX];

  if (!key_path) {
    throw std::runtime_error("$SSL_KEY not defined.");
  }
  if (!cert_path) {
    throw std::runtime_error("$SSL_CERT not defined.");
  }
  if (!root_path) {
    throw std::runtime_error("$SSL_ROOT_CERT not defined.");
  }
  if (!json_path) {
    json_path = "";
  }
  InitFields(key_path, cert_path, root_path, json_path);
}

corenode::SslKeyCert::SslKeyCert(
    const std::string& key_path,
    const std::string& cert_path,
    const std::string& root_path,
    const std::string& json_path) {
  InitFields(key_path, cert_path, root_path, json_path);
}

void corenode::SslKeyCert::InitFields(
    const std::string& key_path,
    const std::string& cert_path,
    const std::string& root_path,
    const std::string& json_path) {
  char resolved_path[PATH_MAX];

  char * found = realpath(key_path.c_str(), resolved_path);
  if (found == NULL) {
    throw std::runtime_error("SSL secret key file not found at specified path.");
  }
  ReadFile(resolved_path, key);

  found = realpath(cert_path.c_str(), resolved_path);
  if (found == NULL) {
    throw std::runtime_error("SSL certificate file not found at specified path.");
  }
  ReadFile(resolved_path, cert);

  found = realpath(root_path.c_str(), resolved_path);
  if (found == NULL) {
    throw std::runtime_error("SSL root CA certificate file not found at specified path.");
  }
  ReadFile(resolved_path, root);

  found = realpath(json_path.c_str(), resolved_path);
  if (found == NULL) {
    client_id_path.assign("");

  } else {
    client_id_path.assign(resolved_path);
    std::string client_id_contents;
    ReadFile(client_id_path, client_id_contents);
    client_id_json = nlohmann::json::parse(client_id_contents);
  }
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

nlohmann::json corenode::SslKeyCert::RequestOAuthToken() {
  if (client_id_path.empty()) {
    throw std::runtime_error("No OAuth2 client ID JSON specified.");
  }

  if (OAUTH2_CLI_EXE.compare("NULL") == 0) {
    throw std::runtime_error("OAUTH2_CLI tool not defined");
  }

  if (oauth_token != NULL) {
    return GetOAuthToken();
  }

  std::string combined_command(OAUTH2_CLI_EXE
      + " " + GetClientIdJsonPath()
      + " envtrackernode");
  std::cout << "oauth2_cli tool specified: " << combined_command << std::endl;
  std::cout << std::endl;

  // Executable oauth2_cli application with passed in CLIENT_SECRET_JSON.
  FILE* fd = popen(combined_command.c_str(), "r");
  std::array<char, 128> buffer;
  std::string result;

  while(!feof(fd)) {
    if (fgets(buffer.data(), 128, fd) != NULL) {
      result.append(buffer.data());
      std::cout << buffer.data();
    }
  }
  std::cout << std::endl;
  pclose(fd);

  // Extract OAuth2 JSON token from oauth2_cli application output.
  std::regex oauth2_token_regex(
      "(.|\n)+CREDENTIALS_START\n(.+)\nCREDENTIALS_END", std::regex::extended);
  std::smatch matches;

  if (std::regex_search(result, matches, oauth2_token_regex) == 0) {
    throw std::runtime_error("No OAuth2 token found.");
  }

  // Convert OAuth2 JSON string to JSON object.
  std::string cleaned_str(matches[2].str());
  oauth_token = nlohmann::json::parse(cleaned_str);
  return oauth_token;
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

void corenode::SslKeyCert::ReplaceAll(std::string& str, const std::string& from, const std::string& to) {
  if (from.empty()) {
    return;
  }
  size_t pos = 0;
  while((pos = str.find(from, pos)) != std::string::npos) {
    str.replace(pos, from.length(), to);
    pos += to.length();
  }
}

std::string corenode::SslKeyCert::GetKey() {
  return std::string(key);
}

std::string corenode::SslKeyCert::GetCert() {
  return std::string(cert);
}

std::string corenode::SslKeyCert::GetRoot() {
  return std::string(root);
}

std::string corenode::SslKeyCert::GetClientIdJsonPath() {
  return std::string(client_id_path);
}

nlohmann::json corenode::SslKeyCert::GetClientIdJson() {
  return nlohmann::json::parse(client_id_json.dump());
}

void corenode::SslKeyCert::SetOAuthToken(const std::string& token) {
  oauth_token = {
    {"token", token},
  };
}

nlohmann::json corenode::SslKeyCert::GetOAuthToken() {
  if (oauth_token == NULL) {
    return RequestOAuthToken();
  }
  return nlohmann::json::parse(oauth_token.dump());
}
