#/bin/bash

docker build -t short-link:latest -f ./Dockerfile --no-cache .

docker stop short-link

docker rm short-link

docker run -d  \
  --name short-link \
  --link mysql  \
  -u root \
  -e DATABASE_URL=mysql://root:123456@mysql:3306/short_link \
  -p 9222:8008 \
  -v ./logs:/logs \
  short-link:latest