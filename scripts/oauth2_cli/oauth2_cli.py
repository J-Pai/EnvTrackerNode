#!/usr/bin/env python3
#cython: language_level=3

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

commandline_mode = False;
app = flask.Flask(__name__)
port = 8080

if len(sys.argv) < 2:
    print('Please specify the path to the client secret JSON.')
    sys.exit(1)
elif len(sys.argv) < 3:
    print('No flask secret key as the second argument. Using commandline mode.')
    commandline_mode = True
else:
    app.secret_key = sys.argv[2]
json_file = os.path.abspath(sys.argv[1])
print('Using client secret JSON: {}'.format(json_file))

@app.route('/auth')
def auth():
    authorization_url, state, _ = generate_authorization_url(
        'https://localhost:{}/oauth2'.format(port))
    flask.session['state'] = state
    return flask.redirect(authorization_url)

@app.route('/oauth2')
def oauth2():
    state = flask.session['state']

    flow = google_auth_oauthlib.flow.Flow.from_client_secrets_file(
        json_file, scopes=SCOPES, state=state)
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
    return flask.redirect('https://www.google.com')

def open_browser():
    webbrowser.open_new('https://localhost:{}/auth'.format(port));

def generate_authorization_url(redirect_uri):
    flow = google_auth_oauthlib.flow.Flow.from_client_secrets_file(json_file, SCOPES)

    flow.redirect_uri = redirect_uri

    authorization_url, state = flow.authorization_url(
        access_type='offline',
        prompt='consent',
        include_granted_scopes='true')

    return authorization_url, state, flow

def credentials_to_dict(credentials):
    return {'token': credentials.token,
            'refresh_token': credentials.refresh_token,
            'token_uri': credentials.token_uri,
            'client_id': credentials.client_id,
            'client_secret': credentials.client_secret,
            'scopes': credentials.scopes}

def get_open_port():
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(('', 0))
        s.listen(1)
        port = s.getsockname()[1]
        return port

if __name__ == '__main__':
    if commandline_mode:
        authorization_url, _, flow = generate_authorization_url('urn:ietf:wg:oauth:2.0:oob')
        print('Please enter the following URL into a browser: \n')
        print('{}\n'.format(authorization_url))
        code = input('Please input the code: ')
        flow.fetch_token(code=code)
        credentials = flow.credentials
        credentials_json = credentials_to_dict(credentials)
        print(credentials_json)
    else:
        Timer(1, open_browser).start()
        port = get_open_port()
        app.run('localhost', port, ssl_context='adhoc')
