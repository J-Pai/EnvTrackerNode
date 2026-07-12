# Forwarding Content

See [PiKVM](pikvm.md) for more information on reverse proxies.

## TLS Certificate Location

```shell
/etc/pki/nginx/server.crt

-r--r--r--.  1 root nginx 623 Jul 12 22:52 server.crt

/etc/pki/nginx/private/server.key

-r--r-----. 1 root nginx 302 Jul 12 22:52 private/server.key
```

## SELinux

```shell
sudo grep "denied" /var/log/audit/audit.log

audit2allow -M local << _EOF_
(All audit denied lines)
_EOF_

sudo semodule -i local.pp
```

May require more then 1 line for this.

## Reverse Proxy

For the proxy_pass address, prefer an IP address.

```shell
    server {
        listen       443 ssl http2;
        listen       [::]:443 ssl http2;
        server_name  _;

        ssl_certificate "/etc/pki/nginx/server.crt";
        ssl_certificate_key "/etc/pki/nginx/private/server.key";
        ssl_ciphers PROFILE=SYSTEM;
        ssl_prefer_server_ciphers on;

        location /env {
            rewrite ^/env$ / break;
            rewrite ^/env\?(.*)$ ?$1 break;
            rewrite ^/env/(.*)$ /$1 break;
            proxy_pass http://bigbox:3000;

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
