# PubNub WANBus
> Bring NATS, Kafka, Redis and RabbitMQ to the real world.

Add push notifications and streaming events to Mobile and Web clients
based on a NATs subjects.
Works with Kafka, Redis, RabbitMQ and more.
Easy drop-in sidecar.
Dashboard management page included.

![WANBus](https://repository-images.githubusercontent.com/178954890/dcda8900-6ce9-11e9-9cc5-a6b7b476ad65)

This application is written in Rust.
The application will listen on your data channels specified by runtime
and deploytime configuration.
Built-in security model allows encyrpted data in motion using 2048bit TLS.
Access management allows you to control
who can read/write on your message bus.

## Dashboard Mockups

![](https://i.imgur.com/BjC73ZN.png)

![](https://i.imgur.com/qP6nNBr.png)

## Build and Run with Docker Container

Production runtime Alpine image size is 6MB.

```shell
docker build --cpuset-cpus 3 -t nats-wanbus .
```

```shell
docker run .................
    -e PUBNUB_SUBKEY=demo
    -e PUBNUB_PUBKEY=demo
    -e PUBNUB_SECKEY=demo
    -e NATS_SUBJECTS=* ## comma delimited list of subjects to sync
    -e NATS_HOST=nats
    -e NATS_USER=
    -e NATS_PASSWORD=
    -e NATS_AUTH_TOKEN=
```

## Test Runtime with Docker Compose

Easily test with `docker-compose`.

```shell
cd nats
docker-compose up
```


## Deploying Sidecar in Kubernetes (K8s)

...
Add Secrets to secret store.
Add YAML sidecar

## Deploying Docker Compose

```shell
docker run -e "NATS_HOST=0.0.0.0:4222" -e "NATS_CHANNELS=a,b,c" nats-wanbus
```

##  Easy NATS Terminal Test

Publish Messages in a Loop.

```shell
while true;
    do (printf "PUB demo 5\r\nKNOCK\r\n"; sleep 0.4) | nc 0.0.0.0 4222;
done
```

Subscribe to these messages in another termianl window.

```shell
while true;
    do (printf "SUB demo 1\r\n"; sleep 60) | nc 0.0.0.0 4222;
done
```

```shell
(printf "SUB FOO 1\r\n"; sleep 5) | nc 0.0.0.0 4222 &
(printf "PING\r\n";                        sleep 0.4; \
 printf "CONNECT {\"verbose\":false}\r\n"; sleep 0.4; \
 printf "SUB BAR 1\r\n";                   sleep 0.4; \
 printf "PING\r\n";                        sleep 0.4; \
 printf "PUB FOO 11\r\nKNOCK KNOCK\r\n";   sleep 0.4; \
 printf "PING\r\n";                        sleep 0.4; \
 printf "PUB BAR 11\r\nKNOCK KNOCK\r\n";   sleep 0.4; \
 printf "PING\r\n";                        sleep 0.4; \
) | nc 0.0.0.0 4222 
```

## Reference Links

[https://hub.docker.com/nats](https://hub.docker.com/_/nats)


## TODOs List

 - TODO - add a STATS thread that collects counters and sends them over a PubNub channel.
 -      - this allows for a dashboard to see that status of the container.
 - TODO - multiple channels NATS - https://doc.rust-lang.org/std/sync/mpsc/#examples
 -      - Multple Channels support (NATS)
 - TODO - read line one last time to get response body
 - TODO - TLS on PubNub
 -      - https://docs.rs/native-tls/0.2.2/native_tls/
 - TODO - JSON decode PubNub response GET TIMETOKEN return Result
 - TODO - JSON
 - TODO - Secret Key Signing on Publish
 - TODO - agent and sec key PubNub struct
 - TODO - &str vs String?
 - TODO - pnsdk=nats-to-mobile / nats-wanbus
 - TODO - CONNECT string for NATS
 - TODO - AsyncAwait & EPOLL ( https://jsdw.me/posts/rust-asyncawait-preview/  )
 - TODO - socket hangout needs to reconnect
 - TODO - error handeling ( unwarp() )
 - TODO - error enum for unrecoverables
 - TODO - recover from disconnect or restart?
 - TODO - http parser?
 - TODO - pubnub.subscribe via /v2/stream endpoint
 - TODO - /// Rust DOC comments?
 - TODO - ENV vars for runtime config ( pub/sub/sec/origin/channel-list )
 - TODO - add more tests!!!!!!!
 - TODO - remove all `println!()` except the system log stdout
 - TODO - dashboard
