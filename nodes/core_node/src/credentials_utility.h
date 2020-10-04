#ifndef ENVTRACKERNODE_CORENODE_CREDENTIALS_UTILITY_H_
#define ENVTRACKERNODE_CORENODE_CREDENTIALS_UTILITY_H_

#include <fstream>
#include <iostream>
#include <linux/limits.h>
#include <memory>
#include <regex>
#include <sstream>
#include <stdexcept>
#include <string>

#include <grpcpp/grpcpp.h>
#include <nlohmann/json.hpp>

#define STR(x) #x
#define XSTR(x) STR(x)

namespace corenode {
class CredentialsUtility final {
  public:
    /**
     * Creates an {@link corenode::CredentialsUtility} object which contains the GCP
     * project client information and supporting SSL documents.
     *
     * This constructor auto fills the SSL documents and client information
     * using the following environment variables:
     * <ul>
     *  <li> SSL_KEY
     *  <li> SSL_CERT
     *  <li> SSL_ROOT_CERT
     *  <li> CLIENT_SECRET_JSON
     * </ul>
     */
    CredentialsUtility();

    /**
     * Creates an {@link corenode::CredentialsUtility} object which contains the GCP
     * project client information and supporting SSL documents.
     *
     * @param key_path Path to SSL client secret key file.
     * @param cert_path Path to SSL client certificate file.
     * @parma root_path Path to SSL root CA certificate file.
     * @param json_path Path to GCP client secret ID JSON file.
     */
    CredentialsUtility(
        const std::string& key_path,
        const std::string& cert_path,
        const std::string& root_path,
        const std::string& json_path);

    /**
     * Creates an {@link corenode::CredentialsUtility} object which contains the GCP
     * project client information and supporting SSL documents.
     *
     * @param env_json_path Path to GCP client secret ID JSON file.
     */
    CredentialsUtility(const std::string& env_json_path);

    /**
     * Generates an {@link grpc::ServerCredentials} object.
     * Uses the stored SSL key, certificate, and root CA.
     */
    std::shared_ptr<grpc::ServerCredentials> GenerateServerCredentials();

    /**
     * Generates an {@link grpc::ServerCredentials} object.
     * Uses the stored SSL key, certificate, and root CA.
     */
    std::shared_ptr<grpc::ChannelCredentials> GenerateChannelCredentials();

    /**
     * Requests an Google OAuth2 token using the oauth2_cli application.
     *
     * @return {@link nlohmann::json}
     */
    nlohmann::json RequestOAuthToken();

    /**
     * Extracts an argument value from the commandline arguments.
     *
     * @param arg_name Flag argument name (must be in the format --arg_name=arg_value).
     * @param default_value Default value to return if flag not found.
     * @param argc Number of argument strings.
     * @param argv List of argument strings.
     * @return value associated with flag.
     */
    static std::string GetFlagValue(
        const std::string& arg_name,
        const std::string& default_value,
        int argc, char** argv);

    std::string GetKey();
    std::string GetCert();
    std::string GetRoot();
    std::string GetClientIdJsonPath();
    nlohmann::json GetClientIdJson();
    void SetOAuthToken(const std::string& token);
    nlohmann::json GetOAuthToken();

  private:
    std::string key;
    std::string cert;
    std::string root;
    std::string client_id_path;
    nlohmann::json client_id_json;
    nlohmann::json oauth_token;
    nlohmann::json environment_json;

    void InitFields(const std::string& key_path, const std::string& cert_path,
        const std::string& root_path, const std::string& json_path);

    /**
     * Replaces every occurance of the string variable "from" with the string
     * variable "to".
     *
     * @param str string to do that needs replacement.
     * @param from target string to search for.
     * @param to string to replace target string with.
     */
    void ReplaceAll(std::string& str, const std::string& from, const std::string& to);

    void ReadFile(const std::string&, std::string&);

    /**
     * Contains the path to the oauth2_cli tool that can be used to obtain a
     * Google OAuth2 access token.
     */
    const std::string OAUTH2_CLI_EXE = XSTR(OAUTH2_CLI);

    const char* HOME = std::getenv("HOME");
};
} // namespace corenode

#endif // ENVTRACKERNODE_CORENODE_CREDENTIALS_UTILITY_H_
