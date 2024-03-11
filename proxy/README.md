# Kupo Proxy

This proxy will allow Kupo to be accessed externally.

## Environment

| Key             | Value            |
| --------------- | ---------------- |
| PROXY_ADDR      | 0.0.0.0:5000     |
| PROXY_NAMESPACE |                  |
| PROMETHEUS_ADDR | 0.0.0.0:9090     |
| SSL_CRT_PATH    | /localhost.crt   |
| SSL_KEY_PATH    | / localhost.key  |
| KUPO_PORT       |                  |
| KUPO_DNS        | INTERNAL K8S DNS |


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
