#!/bin/bash

if [[ -f /sys/class/thermal/thermal_zone0/temp ]]; then
  TEMP=$(cat /sys/class/thermal/thermal_zone0/temp)
  echo "scale=3; ${TEMP}/1000.0" | bc
else
  echo "100.0"
fi
