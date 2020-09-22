#!/usr/bin/env python3

import google.oauth2.credentials
import google_auth_oauthlib.flow

import flask
import logging
import os
import requests
import socket
import sys
from threading import Timer
import webbrowser

log = logging.getLogger('werkzeug')
log.setLevel(logging.ERROR)

SCOPES = ['openid',
          'https://www.googleapis.com/auth/userinfo.profile',
          'https://www.googleapis.com/auth/userinfo.email']

file_dir = os.path.abspath(os.path.dirname(__file__))
json_files = [json_file for json_file in os.listdir(file_dir) if json_file.endswith('.json')]

if len(json_files) == 0:
    print("Client JSON not found at {}.".format(file_dir))
    sys.exit(1)

if len(sys.argv) == 1:
    print("Please specify a flask secret key as the first argument.")
    sys.exit(2)

app = flask.Flask(__name__)
app.secret_key = sys.argv[1]
credentials = {}
port = 8080

@app.route('/auth')
def auth():
    flow = google_auth_oauthlib.flow.Flow.from_client_secrets_file(json_files[0], SCOPES)

    flow.redirect_uri = "https://localhost:{}/oauth2".format(port)

    authorization_url, state = flow.authorization_url(
        access_type='offline',
        include_granted_scopes='true')

    flask.session['state'] = state

    return flask.redirect(authorization_url)

@app.route('/oauth2')
def oauth2():
    state = flask.session['state']

    flow = google_auth_oauthlib.flow.Flow.from_client_secrets_file(
        json_files[0], scopes=SCOPES, state=state)
    flow.redirect_uri = flask.url_for('oauth2', _external=True)

    authorization_response = flask.request.url
    flow.fetch_token(authorization_response=authorization_response)

    credentials = flow.credentials
    flask.session['credentials'] = credentials_to_dict(credentials)

    shutdown = flask.request.environ.get('werkzeug.server.shutdown')
    if shutdown is None:
        raise RuntimeError('Not running with the Werkzeug Server')
    print(flask.session['credentials'])
    shutdown()
    return flask.redirect("https://www.google.com")

def open_browser():
    webbrowser.open_new('https://localhost:{}/auth'.format(port));

def credentials_to_dict(credentials):
    return {'token': credentials.token,
            'refresh_token': credentials.refresh_token,
            'token_uri': credentials.token_uri,
            'client_id': credentials.client_id,
            'client_secret': credentials.client_secret,
            'scopes': credentials.scopes}

def get_open_port():
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.bind(("",0))
    s.listen(1)
    port = s.getsockname()[1]
    s.close()
    return port

if __name__ == '__main__':
    Timer(1, open_browser).start()
    port = get_open_port()
    app.run('localhost', port, ssl_context='adhoc')