#ifndef ENVTRACKERNODE_CORENODE_OAUTH2_TOKEN_PROCESSOR_H_
#define ENVTRACKERNODE_CORENODE_OAUTH2_TOKEN_PROCESSOR_H_

#include <grpcpp/grpcpp.h>

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
};
}

#endif // ENVTRACKERNODE_CORENODE_OAUTH2_TOKEN_PROCESSOR_H_
