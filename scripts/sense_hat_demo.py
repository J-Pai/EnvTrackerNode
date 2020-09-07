#!/bin/env python3

from sense_hat import SenseHat
import subprocess
import os

FACTOR = 1.9 # CPU Temperature adjustment factor
sense = SenseHat()
current_file_dir = os.path.dirname(os.path.realpath(__file__))
cpu_temp_process = subprocess.Popen(["%s/check_temp.sh" % current_file_dir],
                                    stdout=subprocess.PIPE,
                                    stderr=subprocess.PIPE)
cpu_temp, stderr = cpu_temp_process.communicate()
cpu_temperature = float(cpu_temp)

temperature = sense.get_temperature()
calibrated_temp = temperature - ((cpu_temperature - temperature) / FACTOR)

temperature_press = sense.get_temperature_from_pressure()
calibrated_temp_press = temperature_press - ((cpu_temperature - temperature_press) / FACTOR)

temp = round(calibrated_temp, 1)
print("Temperature: %s °C" % temp)

temp_pressure = round(calibrated_temp_press, 1)
print("Temperature (pressure): %s °C" % temp_pressure)

humidity = round(sense.get_humidity(), 1)
print("Humidity: %s %%rH" % humidity)

cpu_temp = round(cpu_temperature, 1)
print("CPU Temperature: %s °C" % cpu_temp)

sense.low_light = True
sense.show_message("T:{t} H:{h} CPU_T:{c_t}".format(
    t=temp, h=humidity, c_t=cpu_temp))
sense.low_light = False
