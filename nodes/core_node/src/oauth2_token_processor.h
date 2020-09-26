#ifndef ENVTRACKERNODE_CORENODE_OAUTH2_TOKEN_PROCESSOR_H_
#define ENVTRACKERNODE_CORENODE_OAUTH2_TOKEN_PROCESSOR_H_

#include <curlpp/cURLpp.hpp>
#include <curlpp/Easy.hpp>
#include <curlpp/Option.hpp>
#include <grpcpp/grpcpp.h>
#include <nlohmann/json.hpp>

#include "ssl_key_cert.h"

namespace corenode {
/**
 * Intercepts gRPC calls and validates the Google OAuth2 bearer token.
 */
class OAuth2TokenProcessor final : public grpc::AuthMetadataProcessor {
  public:
    OAuth2TokenProcessor(const std::shared_ptr<const corenode::SslKeyCert> ssl_key_cert);

    /**
     * Processes the gRPC authorization metadata and determines if the Google
     * OAuth2 bearer token is valid and associated with a registered user.
     */
    grpc::Status Process(
        const InputMetadata& auth_metadata,
        grpc::AuthContext* context,
        OutputMetadata* consumed_auth_metadata,
        OutputMetadata* response_metadata) override;
  private:
    const std::shared_ptr<const corenode::SslKeyCert> ssl_key_cert;
    std::map<std::string, std::string> tokens;

    /**
     * Makes a request to the Google OAuth2 tokeninfo Endpoint to verify the
     * validity of the OAuth2 access token.
     *
     * Endpoint URI: https://oauth2.googleapis.com/tokeninfo?id_token=$BEARER_TOKEN
     *
     * @param token OAuth2 access token.
     * @return {@link nlohmann::json} of body of endpoint reponse.
     */
    nlohmann::json get_token_info(const std::string& token);
};
}

#endif // ENVTRACKERNODE_CORENODE_OAUTH2_TOKEN_PROCESSOR_H_
