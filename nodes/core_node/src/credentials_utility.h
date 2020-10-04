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

#include <bsoncxx/stdx/make_unique.hpp>
#include <grpcpp/grpcpp.h>
#include <mongocxx/instance.hpp>
#include <mongocxx/pool.hpp>
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
     * Connects to MongoDB database using configured credentials.
     */
    void SetupMongoConnection();

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
    mongocxx::pool::entry GetMongoClient();

  private:
    std::string key_;
    std::string cert_;
    std::string root_;
    std::string client_id_path_;
    std::unique_ptr<mongocxx::instance> instance_ = nullptr;
    std::unique_ptr<mongocxx::pool> pool_ = nullptr;
    nlohmann::json mongo_connection_;
    nlohmann::json client_id_json_;
    nlohmann::json oauth_token_;
    nlohmann::json environment_json_;

    void InitFields(
        const std::string& key_path,
        const std::string& cert_path,
        const std::string& root_path,
        const std::string& json_path,
        const nlohmann::json& mongo);

    /**
     * Replaces every occurance of the string variable "from" with the string
     * variable "to".
     *
     * @param str string to do that needs replacement.
     * @param from target string to search for.
     * @param to string to replace target string with.
     */
    void ReplaceAll(std::string& str, const std::string& from, const std::string& to);

    /**
     * Parses the file located at the path filename and dumps it's contents into
     * data.
     *
     * @param filename Path to file.
     * @return String with the contents of the files.
     */
    std::string ReadFile(const std::array<char, PATH_MAX>& filename);

    /**
     * Contains the path to the oauth2_cli tool that can be used to obtain a
     * Google OAuth2 access token.
     */
    const std::string kOAuth2CLI = XSTR(OAUTH2_CLI);

    const char* kHome = std::getenv("HOME");
};
} // namespace corenode

#endif // ENVTRACKERNODE_CORENODE_CREDENTIALS_UTILITY_H_
