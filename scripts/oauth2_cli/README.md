# Google OAuth2 Token CLI Utility
Commandline utility for generating Google OAuth2 tokens for use in other
commandline tools.

From a GCP project's console ([credentials page](https://console.cloud.google.com/apis/credentials)),
create an OAuth2 Client ID and download the client secret JSON into this directory.

```bash
sudo apt install python-venv
cd scripts/oauth2_cli
python -m venv ./venv
source ./venv/bin/activate

python3 -m pip install -r requirements.txt
./oauth2_cli.py $FLASK_SECRET_KEY
```
