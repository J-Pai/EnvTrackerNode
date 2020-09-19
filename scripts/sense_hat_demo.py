#!/usr/bin/env python3

import os
try:
    from sense_hat import SenseHat
except:
    from sense_emu import SenseHat
import subprocess
import sys
import time

FACTOR = 1.4 # CPU Temperature adjustment factor
LINEAR_FACTOR = 2
sense = SenseHat()

current_file_dir = os.path.dirname(os.path.realpath(__file__))
cpu_temp_process = subprocess.Popen(["%s/check_temp.sh" % current_file_dir],
                                    stdout=subprocess.PIPE,
                                    stderr=subprocess.PIPE)
cpu_temp, stderr = cpu_temp_process.communicate()
cpu_temperature = float(cpu_temp)

temperature = sense.get_temperature()
calibrated_temp = temperature - LINEAR_FACTOR # - ((cpu_temperature - temperature) / FACTOR)

temp = round(calibrated_temp, 1)
temp_f = round(calibrated_temp * 9/5 + 32, 1)
print("Temperature: %s °C - %s °F" % (temp, temp_f))

humidity = round(sense.get_humidity(), 1)
print("Humidity: %s %%rH" % humidity)

cpu_temp = round(cpu_temperature, 1)
print("CPU Temperature: %s °C" % cpu_temp)

print("Waiting for joystick event...")
event = sense.stick.wait_for_event()
print("The joystick was {} {}".format(event.action, event.direction))

sense.show_message("T:{t} H:{h}".format(
    t=temp_f, h=humidity))

orientation = sense.get_orientation()
print("p: {pitch}, r: {roll}, y: {yaw}".format(**orientation))

north = sense.get_compass()
print("North: %s" % north)

gyro_only = sense.get_gyroscope()
print("p: {pitch}, r: {roll}, y: {yaw}".format(**gyro_only))

accel_only = sense.get_accelerometer()
print("p: {pitch}, r: {roll}, y: {yaw}".format(**accel_only))
