#ifndef ENVTRACKERNODE_CORENODE_OAUTH2_TOKEN_PROCESSOR_H_
#define ENVTRACKERNODE_CORENODE_OAUTH2_TOKEN_PROCESSOR_H_

#include <curlpp/cURLpp.hpp>
#include <curlpp/Easy.hpp>
#include <curlpp/Infos.hpp>
#include <curlpp/Options.hpp>
#include <grpcpp/grpcpp.h>
#include <nlohmann/json.hpp>

#include "credentials_utility.h"

namespace corenode {
/**
 * Intercepts gRPC calls and validates the Google OAuth2 bearer token.
 */
class OAuth2TokenProcessor final : public grpc::AuthMetadataProcessor {
  public:
    OAuth2TokenProcessor(std::shared_ptr<corenode::CredentialsUtility> ssl_key_cert);

    /**
     * Processes the gRPC authorization metadata and determines if the Google
     * OAuth2 bearer token is valid and associated with a registered user.
     */
    grpc::Status Process(
        const InputMetadata& auth_metadata,
        grpc::AuthContext* context,
        OutputMetadata* consumed_auth_metadata,
        OutputMetadata* response_metadata) override;

    /**
     * Makes a request to the Google OAuth2 tokeninfo Endpoint to verify the
     * validity of the OAuth2 access token.
     *
     * Endpoint URI: https://oauth2.googleapis.com/tokeninfo?id_token=$BEARER_TOKEN
     *
     * @param token OAuth2 access token.
     * @return {@link nlohmann::json} of body of endpoint reponse.
     */
    static nlohmann::json GetTokenInfo(const std::string& token);

  private:
    std::shared_ptr<corenode::CredentialsUtility> ssl_key_cert_;
    std::map<std::string, std::string> tokens_;

    /**
     * Verify that the Google OAuth2 token contains the correct information.
     *
     * @param token_info Token information returned from Google OAuth2 endpoint.
     * @return true if OAuth2 token is valid, false otherwise.
     */
    bool ValidateTokenInfo(const nlohmann::json& token_info);

    static const std::string kTokenInfoEndpoint;
    static const size_t kBearerTextLength = 7;
};
}

#endif // ENVTRACKERNODE_CORENODE_OAUTH2_TOKEN_PROCESSOR_H_
