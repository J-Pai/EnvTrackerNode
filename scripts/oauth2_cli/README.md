# Google OAuth2 Token CLI Utility
Commandline utility for generating Google OAuth2 tokens for use in other
commandline tools.

From a GCP project's console ([credentials page](https://console.cloud.google.com/apis/credentials)),
create an OAuth2 Client ID and download the client secret JSON into this directory.

```bash
cd scripts/oauth2_cli
mkdir -p cmake/build
cd cmake/build
cmake ../..
make
./oauth2_cli $FLASK_SECRET
```
