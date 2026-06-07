events {
    worker_connections 1024;
}

http {
    include {{ bin_dir }}/conf/mime.types;
    default_type application/octet-stream;
    sendfile on;
    keepalive_timeout 65;

    access_log {{ log_dir }}/access.log;
    error_log {{ log_dir }}/error.log;

    server {
        listen {{ port }};
        server_name localhost;

        root {{ www_dir }};
        index index.html index.htm;

        location / {
            try_files $uri $uri/ =404;
        }
    }

    include {{ vhosts_dir }}/*.conf;
}
