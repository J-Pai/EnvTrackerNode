#!/bin/env python3

from sense_hat import SenseHat

sense = SenseHat()

temperature = round(sense.get_temperature(), 1)
print("Temperature: %s C" % temperature)

humidity = round(sense.get_humidity(), 1)
print("Humidity: %s %%rH" % humidity)

sense.show_message("T:{t} H:{h}".format(t=temperature, h=humidity))
