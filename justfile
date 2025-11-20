default:
    @just -l

build-docker-image:
    docker build -t glitchtip-webhook-relay:latest .

build-docker-image-cn:
    docker build --build-arg USE_CN_MIRROR=1 -t glitchtip-webhook-relay:latest .

