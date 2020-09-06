#!/bin/bash

TEMP=$(cat /sys/class/thermal/thermal_zone0/temp)
echo "scale=3; ${TEMP}/1000.0" | bc
