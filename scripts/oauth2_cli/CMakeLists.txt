cmake_minimum_required(VERSION 3.16.3)

project(OAuth2CLI)

find_package(Python3 REQUIRED)

add_custom_command(
  OUTPUT venv
  COMMAND python3
  ARGS -m venv venv)

add_custom_command(
  OUTPUT venv.stamp
  DEPENDS venv requirements.txt
  COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_SOURCE_DIR}/requirements.txt requirements.txt
  COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_SOURCE_DIR}/*.json .
  COMMAND ./venv/bin/pip install -r requirements.txt)

add_custom_target(oauth2_cli
  ALL
  DEPENDS venv.stamp
  COMMAND ./venv/bin/pyinstaller --onefile ${CMAKE_CURRENT_SOURCE_DIR}/oauth2_cli.py
  COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_BINARY_DIR}/dist/oauth2_cli oauth2_cli)

add_custom_target(run
  COMMAND ${CMAKE_CURRENT_BINARY_DIR}/venv/bin/python ${CMAKE_CURRENT_SOURCE_DIR}/oauth2_cli.py
  DEPENDS oauth2_cli
  WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR})
