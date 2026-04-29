# Forwarding Content

We insert a `x-real-base` header field so the static content web server
can overwrite the base html file with the correct redirects.

See [PiKVM](pikvm.md) for more information on reverse proxies.

## Reverse Proxy

```shell
        location /env {
            rewrite ^/env$ / break;
            rewrite ^/env\?(.*)$ ?$1 break;
            rewrite ^/env/(.*)$ /$1 break;
            proxy_pass http://bigbox:3000;

            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Real-Base /env;
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
```
