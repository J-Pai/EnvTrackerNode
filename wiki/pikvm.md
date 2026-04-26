# PiKVM

## Self-Signed Certificates Location

Certificates for kvmd-nginx is located at:

```shell
$ ls -al /etc/kvmd/nginx/ssl
total 16
drwxr-xr-x 2 root       root       4096 Aug 18  2024 .
drwxr-xr-x 3 root       root       4096 Apr 18 07:53 ..
-r-------- 1 kvmd-nginx kvmd-nginx  867 Apr 26 01:09 server.crt
-r-------- 1 kvmd-nginx kvmd-nginx  302 Apr 26 01:09 server.key
```

## Reverse Proxy

Proxying PiKVM traffic through another server via nginx.

- Configuration: /etc/nginx/nginx.conf

```shell
    # pikvm
    server {
        listen       443 ssl http2;
        listen       [::]:443 ssl http2;
        server_name  _;

        ssl_certificate "/etc/pki/nginx/server.crt";
        ssl_certificate_key "/etc/pki/nginx/private/server.key";
        ssl_ciphers PROFILE=SYSTEM;
        ssl_prefer_server_ciphers on;

        location / {
            proxy_pass https://pikvm;

            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Scheme $scheme;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_set_header X-Forwarded-Port $server_port;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;

            # For some handles (like MJPEG) buffering should be disabled
            postpone_output 0;
            proxy_buffering off;
            proxy_ignore_headers X-Accel-Buffering;

            # Some handles (ends with /ws) are WebSockets
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_connect_timeout 7d;
            proxy_send_timeout 7d;
            proxy_read_timeout 7d;

            # Some other handles requires big POST payload
            client_max_body_size 0;
            proxy_request_buffering off;
        }
    }
```

- SELinux

```shell
sudo setsebool -P httpd_can_network_connect 1
```

- Systemd Service

You may need this so nginx will keep trying to resolve the pikvm tailscale IP instead
of shutting down immediately after a reboot.

```shell
[Service]
Restart=always
RestartSec=5s
```
