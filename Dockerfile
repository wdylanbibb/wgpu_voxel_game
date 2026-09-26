FROM nginxinc/nginx-unprivileged:1.29-alpine

COPY --chown=101:101 dist/ /usr/share/nginx/html/
