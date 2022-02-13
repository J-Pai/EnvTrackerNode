#include "oauth2_token_processor.h"

const std::string corenode::OAuth2TokenProcessor::kTokenInfoEndpoint =
    "https://www.googleapis.com/oauth2/v3/tokeninfo?access_token=%s";

corenode::OAuth2TokenProcessor::OAuth2TokenProcessor(
    std::shared_ptr<corenode::CredentialsUtility> ssl_key_cert)
    : ssl_key_cert_(ssl_key_cert) {}

grpc::Status corenode::OAuth2TokenProcessor::Process(
    const InputMetadata& auth_metadata,
    grpc::AuthContext* context,
    OutputMetadata* consumed_auth_metadata,
    OutputMetadata* response_metadata) {
  // DEBUG_INFO_START
  std::cout << "Using token processor..." << std::endl;
  std::multimap<grpc::string_ref, grpc::string_ref>::const_iterator itr;
  for (itr = auth_metadata.begin(); itr != auth_metadata.end(); ++itr) {
    std::cout << itr->first << "," << itr->second << std::endl;
  }
  // DEBUG_INFO_END

  // Determine intercepted method
  std::multimap<grpc::string_ref, grpc::string_ref>::const_iterator path =
    auth_metadata.find(":path");
  if (path == auth_metadata.end()) {
    return grpc::Status(grpc::StatusCode::INTERNAL, "Unknown path.");
  }

  // Verify request contains access token.
  std::multimap<grpc::string_ref, grpc::string_ref>::const_iterator token =
    auth_metadata.find("authorization");
  if (token == auth_metadata.end()) {
    return grpc::Status(grpc::StatusCode::UNAUTHENTICATED, "Missing access token.");
  }

  std::string raw(token->second.data());
  std::string bearer_token(raw.substr(
        kBearerTextLength, token->second.length() - kBearerTextLength));

  nlohmann::json token_info;
  try {
    token_info = GetTokenInfo(bearer_token);
  } catch (const std::runtime_error& error) {
    return grpc::Status(grpc::StatusCode::UNAUTHENTICATED, error.what());
  }

  if (!ValidateTokenInfo(token_info)) {
    return grpc::Status(
        grpc::StatusCode::UNAUTHENTICATED, "Invalid access token.");
  }

  return grpc::Status::OK;
}

/**
 * Suppresses the curl output when used as the WriteFunction callback.
 */
size_t curl_write_function(void* buffer, size_t size, size_t nmemb) {
  return size * nmemb;
}

nlohmann::json corenode::OAuth2TokenProcessor::GetTokenInfo(
    const std::string& token) {
  size_t url_length = token.length() + kTokenInfoEndpoint.length();
  char buffer[url_length];
  snprintf(buffer, url_length, kTokenInfoEndpoint.c_str(), token.c_str());

  std::ostringstream response;

  curlpp::Easy request;
  request.setOpt<curlpp::options::Url>(std::string(buffer));
  request.setOpt<curlpp::options::WriteFunction>(curl_write_function);
  request.setOpt<curlpp::options::WriteStream>(&response);
  request.perform();

  long http_code = curlpp::Infos::ResponseCode::get(request);

  if (http_code != 200) {
    throw std::runtime_error(std::string(response.str()));
  }

  return nlohmann::json::parse(response.str());
}

bool corenode::OAuth2TokenProcessor::ValidateTokenInfo(
    const nlohmann::json& token_info) {
  std::cout << token_info << std::endl;
  nlohmann::json client_info(ssl_key_cert_->GetClientIdJson()["installed"]);
  std::cout << client_info << std::endl;

  // Verify that the token was obtained from the expected OAuth2 client.
  std::string client_id(client_info["client_id"]);
  std::string token_aud(token_info["aud"]);
  if (client_id.compare(token_aud) != 0) {
    return false;
  }

  // TODO: Verify that the user is authorized to access this backend.
  // Recommended to use std::future and std::async to make request against
  // external backend.

  return true;
}
