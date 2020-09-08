#!/bin/env python3

from sense_hat import SenseHat
import subprocess
import sys
import os

FACTOR = 1.4 # CPU Temperature adjustment factor
sense = SenseHat()
current_file_dir = os.path.dirname(os.path.realpath(__file__))
cpu_temp_process = subprocess.Popen(["%s/check_temp.sh" % current_file_dir],
                                    stdout=subprocess.PIPE,
                                    stderr=subprocess.PIPE)
cpu_temp, stderr = cpu_temp_process.communicate()
cpu_temperature = float(cpu_temp)

temperature = sense.get_temperature_from_pressure()
calibrated_temp = temperature - ((cpu_temperature - temperature) / FACTOR)

temp = round(calibrated_temp, 1)
temp_f = round(calibrated_temp * 9/5 + 32, 1)
print("Temperature: %s °C - %s °F" % (temp, temp_f))

humidity = round(sense.get_humidity(), 1)
print("Humidity: %s %%rH" % humidity)

cpu_temp = round(cpu_temperature, 1)
print("CPU Temperature: %s °C" % cpu_temp)

if len(sys.argv) > 1:
    sense.low_light = True
    sense.show_message("T:{t} H:{h}".format(
        t=temp_f, h=humidity))
    sense.low_light = False
