# Environment Tracker Node

### Example Config File

```toml
[settings]
option1 = true
option2 = false

[kasa]

[kasa.smart_plug]
host = "123.456.789.123"
username = "xxx"
password = "xxx"
```

### Development

```shell
virtualenv venv
. venv/bin/activate
pip install -r requirements.txt
pip freeze -r requirements.txt
```

