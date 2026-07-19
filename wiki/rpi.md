# Raspberry Pi Optimizations

## Disable WIFI PM

- Potential issue with WiFi going to lower power mode and significantly
  impacting connection performance.

```shell
vim /etc/udev/rules.d/81-wifi-powersave.rules
```

```shell
ACTION=="add", SUBSYSTEM=="net", KERNEL=="wl*", RUN+="/usr/bin/iw dev $name set power_save off"
```
