#!/bin/sh

docker build -t n3phtys/aemterliste_rs:latest . || exit 1
docker save n3phtys/aemterliste_rs:latest > image.tar && scp -P 822 ./image.tar gruin@159.69.27.209:/tmp/image.tar && rm image.tar
ssh -p 822 gruin@159.69.27.209 'cd /opt/reservierungen && docker load -i /tmp/image.tar && docker-compose up -d --no-deps'
