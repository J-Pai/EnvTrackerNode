# Google OAuth2 Token CLI Utility
Commandline utility for generating Google OAuth2 tokens for use in other
commandline tools.

From a GCP project's console ([credentials page](https://console.cloud.google.com/apis/credentials)),
create an OAuth2 Client ID and download the client secret JSON to a secure location.

```bash
cd scripts/oauth2_cli
mkdir -p cmake/build
cd cmake/build
cmake ../..
make
sudo make install

oauth2_cli /path/to/client/secret/secret.json $FLASK_SECRET
```

Specify `$FLASK_SECRET` if you want to automatically request the token. If
`$FLASK_SECRET` is not specified, `oauth2_cli` will require you to input a code
that is generated after the Google login page.
