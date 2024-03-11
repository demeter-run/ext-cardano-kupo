# Kupo Operator

This operator will create a key into the CRD to allow Kupo to be accessed externally.

## Environment

| Key                  | Value                         |
| -------------------- | ----------------------------- |
| ADDR                 | 0.0.0.0:5000                  |
| EXTENSION_SUBDOMAIN  | kupo-m1                       |
| API_KEY_SALT         | kupo-salt                     |
| METRICS_DELAY        | 40                            |
| PROMETHEUS_URL       |                               |
| DCU_PER_FRAME        | preview=5,preprod=5,mainnet=5 |
| DEFAULT_KUPO_VERSION | 2                             |

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
