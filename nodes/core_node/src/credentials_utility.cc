#include "credentials_utility.h"

corenode::CredentialsUtility::CredentialsUtility() {
  const char* key_path = std::getenv("SSL_KEY");
  const char* cert_path = std::getenv("SSL_CERT");
  const char* root_path = std::getenv("SSL_ROOT_CERT");
  const char* json_path = std::getenv("CLIENT_SECRET_JSON");
  const char* mongo_uri = std::getenv("MONGO_URI");
  const char* mongo_user = std::getenv("MONGO_USER");
  const char* mongo_pass = std::getenv("MONGO_PASSWORD");

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

  nlohmann::json mongo;

  if (mongo_uri && mongo_user && mongo_pass) {
    mongo["uri"] = std::string(mongo_uri);
    mongo["user"] = std::string(mongo_user);
    mongo["password"] = std::string(mongo_pass);
  }

  InitFields(key_path, cert_path, root_path, json_path, mongo);
}

corenode::CredentialsUtility::CredentialsUtility(const std::string& env_json_path) {
  std::array<char, PATH_MAX> resolved_path;

  char * found = realpath(env_json_path.c_str(), resolved_path.data());
  if (found == NULL) {
    throw std::runtime_error("Environment JSON file not found at specified path.");
  }
  std::string env_json_contents = ReadFile(resolved_path);
  environment_json_ = nlohmann::json::parse(env_json_contents);

  std::string key_path(environment_json_.contains("ssl_key") ?
      environment_json_["ssl_key"] : "");
  std::string cert_path(environment_json_.contains("ssl_cert") ?
      environment_json_["ssl_cert"] : "");
  std::string root_path(environment_json_.contains("ssl_root_cert") ?
      environment_json_["ssl_root_cert"] : "");
  std::string json_path(environment_json_.contains("client_secret_json") ?
      environment_json_["client_secret_json"] : "");
  nlohmann::json mongo(environment_json_.contains("mongo") ?
      environment_json_["mongo"] : nlohmann::json());

  ReplaceAll(key_path, "${HOME}", kHome);
  ReplaceAll(cert_path, "${HOME}", kHome);
  ReplaceAll(root_path, "${HOME}", kHome);
  ReplaceAll(json_path, "${HOME}", kHome);
  ReplaceAll(key_path, "~", kHome);
  ReplaceAll(cert_path, "~", kHome);
  ReplaceAll(root_path, "~", kHome);
  ReplaceAll(json_path, "~", kHome);

  InitFields(key_path, cert_path, root_path, json_path, mongo);
}

void corenode::CredentialsUtility::InitFields(
    const std::string& key_path,
    const std::string& cert_path,
    const std::string& root_path,
    const std::string& json_path,
    const nlohmann::json& mongo) {
  std::array<char, PATH_MAX> resolved_path;

  char * found = realpath(key_path.c_str(), resolved_path.data());
  if (found == NULL) {
    throw std::runtime_error("SSL secret key file not found at specified path.");
  }
  key_.assign(ReadFile(resolved_path));

  found = realpath(cert_path.c_str(), resolved_path.data());
  if (found == NULL) {
    throw std::runtime_error("SSL certificate file not found at specified path.");
  }
  cert_.assign(ReadFile(resolved_path));

  found = realpath(root_path.c_str(), resolved_path.data());
  if (found == NULL) {
    throw std::runtime_error("SSL root CA certificate file not found at specified path.");
  }
  root_.assign(ReadFile(resolved_path));

  found = realpath(json_path.c_str(), resolved_path.data());
  if (found == NULL) {
    client_id_path_.assign("");
  } else {
    client_id_path_.assign(resolved_path.data());
    std::string client_id_contents;
    client_id_contents.assign(ReadFile(resolved_path));
    client_id_json_ = nlohmann::json::parse(client_id_contents);
  }

  mongo_connection_ = mongo;
}

std::shared_ptr<grpc::ServerCredentials> corenode::CredentialsUtility::GenerateServerCredentials() {
  grpc::SslServerCredentialsOptions::PemKeyCertPair key_cert = {
    key_,
    cert_
  };

  grpc::SslServerCredentialsOptions ssl_opts;
  ssl_opts.pem_root_certs = root_;
  ssl_opts.pem_key_cert_pairs.push_back(key_cert);

  return grpc::SslServerCredentials(ssl_opts);
}

std::shared_ptr<grpc::ChannelCredentials> corenode::CredentialsUtility::GenerateChannelCredentials() {
  grpc::SslCredentialsOptions ssl_opts = {
    root_,
    key_,
    cert_
  };

  return grpc::SslCredentials(ssl_opts);
}

nlohmann::json corenode::CredentialsUtility::RequestOAuthToken() {
  if (client_id_path_.empty()) {
    throw std::runtime_error("No OAuth2 client ID JSON specified.");
  }

  if (kOAuth2CLI.compare("NULL") == 0) {
    throw std::runtime_error("OAUTH2_CLI tool not defined");
  }

  if (!oauth_token_.empty()) {
    return GetOAuthToken();
  }

  std::string combined_command(kOAuth2CLI
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
  oauth_token_ = nlohmann::json::parse(cleaned_str);
  return oauth_token_;
}

void corenode::CredentialsUtility::SetupMongoConnection() {
  if (mongo_connection_.empty()) {
    throw new std::runtime_error("No mongo credentials.");
  }

  if (pool_ != nullptr) {
    return;
  }

  std::unique_ptr<mongocxx::instance> init_instance =
    bsoncxx::stdx::make_unique<mongocxx::instance>();

  std::unique_ptr<mongocxx::pool> init_pool = bsoncxx::stdx::make_unique<mongocxx::pool>(
    mongocxx::uri{
      std::string("mongodb+srv://")
        + std::string(mongo_connection_["user"])
        + ":" + std::string(mongo_connection_["password"])
        + "@" + std::string(mongo_connection_["uri"])
        + "?retryWrites=true&w=majority&tls=true"
    });

  pool_ = std::move(init_pool);
  instance_ = std::move(init_instance);
}

std::string corenode::CredentialsUtility::ReadFile(const std::array<char, PATH_MAX>& filename) {
  std::ifstream file(filename.data(), std::ios::in);
  if (file.is_open()) {
    std::stringstream string_stream;
    string_stream << file.rdbuf();
    file.close();
    return string_stream.str();
  }
  return "";
}

void corenode::CredentialsUtility::ReplaceAll(std::string& str, const std::string& from, const std::string& to) {
  if (from.empty()) {
    return;
  }
  size_t pos = 0;
  while((pos = str.find(from, pos)) != std::string::npos) {
    str.replace(pos, from.length(), to);
    pos += to.length();
  }
}

std::string corenode::CredentialsUtility::GetKey() {
  return std::string(key_);
}

std::string corenode::CredentialsUtility::GetCert() {
  return std::string(cert_);
}

std::string corenode::CredentialsUtility::GetRoot() {
  return std::string(root_);
}

std::string corenode::CredentialsUtility::GetClientIdJsonPath() {
  return std::string(client_id_path_);
}

nlohmann::json corenode::CredentialsUtility::GetClientIdJson() {
  return nlohmann::json::parse(client_id_json_.dump());
}

void corenode::CredentialsUtility::SetOAuthToken(const std::string& token) {
  oauth_token_ = {
    {"token", token},
  };
}

nlohmann::json corenode::CredentialsUtility::GetOAuthToken() {
  if (oauth_token_.empty()) {
    return RequestOAuthToken();
  }
  return nlohmann::json::parse(oauth_token_.dump());
}

std::string corenode::CredentialsUtility::GetFlagValue(
    const std::string& arg_name,
    const std::string& default_value,
    int argc, char** argv) {
  std::string target_str;
  std::string arg_str("--" + arg_name);
  for (int i = 1; i < argc; i++) {
    std::string arg_val = argv[i];
    size_t start_pos = arg_val.find(arg_str);
    if (start_pos < arg_val.length()) {
      if (start_pos != std::string::npos) {
        start_pos += arg_str.size();
        if (arg_val[start_pos] == '=') {
          target_str = arg_val.substr(start_pos + 1);
        } else {
          std::cout << "The only correct argument syntax is --"
            << arg_name << "=" << std::endl;
          exit(1);
        }
      } else {
        std::cout << "The only correct argument syntax is --"
          << arg_name << "=" << std::endl;
        exit(2);
      }
      return target_str;
    }
  }
  return default_value.empty() ? "" : default_value;
}
