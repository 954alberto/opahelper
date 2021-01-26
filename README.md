# opahelper
Helps by downloading a bundle file from Gitlab

## Build binary for linux using docker

```bash
docker run --rm --user "0":"0" -v "$PWD":/usr/src/myapp -w /usr/src/myapp rust:1.47.0 cargo build --release --target-dir=target/linux
```

## Build docker image

```bash
TAG=$(git tag -l | tail -n1);
docker build -t sbpcat/opahelper:${TAG} -f docker/Dockerfile .

```
