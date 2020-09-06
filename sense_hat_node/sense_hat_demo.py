#!/bin/env python3

from sense_hat import SenseHat
import subprocess
import os

FACTOR = 2 # CPU Temperature adjustment factor
sense = SenseHat()
current_file_dir = os.path.dirname(os.path.realpath(__file__))
cpu_temp_process = subprocess.Popen(["%s/../scripts/check_temp.sh" % current_file_dir],
                                    stdout=subprocess.PIPE,
                                    stderr=subprocess.PIPE)
cpu_temp, stderr = cpu_temp_process.communicate()
cpu_temperature = float(cpu_temp)

temperature = sense.get_temperature()
calibrated_temperature = temperature - ((cpu_temperature - temperature) / FACTOR)

temp = round(calibrated_temperature, 1)
print("Temperature: %s °C" % calibrated_temperature)

humidity = round(sense.get_humidity(), 1)
print("Humidity: %s %%rH" % humidity)

print("CPU Temperature: %s °C" % cpu_temperature)

sense.low_light = True
sense.show_message("T:{t} H:{h} CPU_T:{c_t}".format(
    t=temperature, h=humidity, c_t=cpu_temperature))
sense.low_light = False
