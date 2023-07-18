#/bin/bash

docker build -t short-link:latest -f ./Dockerfile.local --no-cache .

docker stop short-link

docker rm short-link


docker run -d  \
  --name short-link \
  --link mysql \
  --link redis \
  -u root \
  -e DATABASE_URL=mysql://root:123456@mysql:3306/short_link \
  -e REDIS_URL=redis://redis:6379 \
  -p 9222:8008 \
  -v "$PWD/application.yaml":/usr/app/application.yaml \
  -v "$PWD/logs":/usr/app/logs \
  short-link:latest