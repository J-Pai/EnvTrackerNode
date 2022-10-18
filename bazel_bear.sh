#!/bin/bash

bear --append bazel --batch build \
  --action_env=LD_PRELOAD=${LD_PRELOAD} \
  --spawn_strategy=local $@
