cmake_minimum_required(VERSION 3.16.3)

project(OAuth2CLI)

find_package(Python3 REQUIRED)

# Create the python3 virtual environment.
add_custom_command(
  OUTPUT venv
  COMMAND python3
  ARGS -m venv venv)

# Install oauth2_cli dependencies into python3 virtual environment.
add_custom_command(
  OUTPUT venv.stamp requirements.txt client_id_secret.json
  DEPENDS venv ${CMAKE_CURRENT_SOURCE_DIR}/requirements.txt
  COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_SOURCE_DIR}/requirements.txt requirements.txt
  COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_SOURCE_DIR}/*.json client_id_secret.json
  COMMAND touch venv.stamp
  COMMAND ./venv/bin/pip install -r requirements.txt)

# Build the oauth2_cli standalone executable.
add_custom_command(
  OUTPUT dist build oauth2_cli.py oauth2_cli.spec __pycache__ oauth2_cli
  DEPENDS venv.stamp ${CMAKE_CURRENT_SOURCE_DIR}/oauth2_cli.py
  COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_SOURCE_DIR}/oauth2_cli.py oauth2_cli.py
  COMMAND ./venv/bin/pyinstaller --onefile oauth2_cli.py
  COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_BINARY_DIR}/dist/oauth2_cli oauth2_cli)

add_custom_target(oauth2_cli_target
  ALL
  DEPENDS oauth2_cli)

add_custom_target(run
  COMMAND ${CMAKE_CURRENT_BINARY_DIR}/venv/bin/python ${CMAKE_CURRENT_SOURCE_DIR}/oauth2_cli.py
  DEPENDS oauth2_cli_target
  WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR})
