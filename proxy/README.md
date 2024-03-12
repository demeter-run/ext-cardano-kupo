# Kupo Proxy

This proxy will allow Kupo to be accessed externally.

## Environment

| Key              | Value                   |
| ---------------- | ----------------------- |
| PROXY_ADDR       | 0.0.0.0:5000            |
| PROXY_NAMESPACE  |                         |
| PROMETHEUS_ADDR  | 0.0.0.0:9090            |
| SSL_CRT_PATH     | /localhost.crt          |
| SSL_KEY_PATH     | /localhost.key          |
| KUPO_PORT        |                         |
| KUPO_DNS         | internal k8s dns        |
| PROXY_TIERS_PATH | path of tiers toml file |

## Rate limit
To define rate limits, it's necessary to create a file with the limiters available that the ports can use. The request limit of each tier can be configured using `second`, `minute`, `hour` and `day`.

```toml
[[tiers]]
name = "0"
second = 1
minute = 10
hour = 100
day = 1000

[[tiers]]
name = "1"
second = 10
hour = 20
```

after configuring, the file path must be set at the env `PROXY_TIERS_PATH`.


## Commands

To generate the CRD will need to execute crdgen

```bash
cargo run --bin=crdgen
```

and execute the operator

```bash
cargo run
```

## Metrics

to collect metrics for Prometheus, an HTTP API will enable the route /metrics.

```
/metrics
```
