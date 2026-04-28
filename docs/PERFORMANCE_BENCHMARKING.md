# Router Metrics Exporter - Performance Benchmarking

This document describes how to benchmark the performance of the router-metrics-exporter under different loads.

## Prerequisites

### Install k6

k6 is a modern load testing tool. Install it from [k6.io](https://k6.io/docs/getting-started/installation/):

**macOS:**
```bash
brew install k6
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6-stable.list
sudo apt-get update
sudo apt-get install k6
```

**Docker:**
```bash
docker run -i grafana/k6 run - < script.js
```

### Start the Metrics Exporter

```bash
cd metrics
cargo run --release
```

The exporter will start on `http://localhost:9090` by default.

## Running Load Tests

### Basic Load Test (30 seconds, 10 VUs, 100 RPS)

```bash
./scripts/load-test.sh
```

### Custom Parameters

```bash
./scripts/load-test.sh [duration] [vus] [rps]
```

**Parameters:**
- `duration` — Test duration (e.g., `30s`, `1m`, `5m`)
- `vus` — Number of virtual users (concurrent connections)
- `rps` — Target requests per second

**Examples:**

```bash
# Light load: 30 seconds, 10 VUs, 100 RPS
./scripts/load-test.sh 30s 10 100

# Medium load: 1 minute, 50 VUs, 500 RPS
./scripts/load-test.sh 1m 50 500

# Heavy load: 5 minutes, 100 VUs, 1000 RPS
./scripts/load-test.sh 5m 100 1000

# Stress test: 10 minutes, 200 VUs, 2000 RPS
./scripts/load-test.sh 10m 200 2000
```

### With Authentication

```bash
export ROUTER_API_KEY="your-secret-key"
./scripts/load-test.sh 30s 10 100
```

### With Custom Base URL

```bash
export BASE_URL="http://example.com:9090"
./scripts/load-test.sh 30s 10 100
```

## Understanding the Results

The load test measures the following metrics:

### Latency Metrics

- **metrics_latency** — Response time for `/metrics` endpoint
- **health_latency** — Response time for `/health` endpoint
- **ready_latency** — Response time for `/ready` endpoint

### Throughput Metrics

- **requests** — Total number of requests sent
- **active_vus** — Number of active virtual users

### Error Metrics

- **errors** — Rate of failed requests (non-2xx responses)

### Thresholds

The test passes if:
- 95th percentile latency < 500ms
- 99th percentile latency < 1000ms
- Error rate < 10%

## Performance Benchmarks

### Baseline (Single Machine, No Load)

| Metric | Value |
|--------|-------|
| Avg Latency | ~5ms |
| P95 Latency | ~10ms |
| P99 Latency | ~20ms |
| Throughput | ~10,000 RPS |
| Error Rate | 0% |

### Light Load (10 VUs, 100 RPS)

| Metric | Value |
|--------|-------|
| Avg Latency | ~10ms |
| P95 Latency | ~25ms |
| P99 Latency | ~50ms |
| Throughput | ~100 RPS |
| Error Rate | 0% |

### Medium Load (50 VUs, 500 RPS)

| Metric | Value |
|--------|-------|
| Avg Latency | ~50ms |
| P95 Latency | ~100ms |
| P99 Latency | ~200ms |
| Throughput | ~500 RPS |
| Error Rate | 0% |

### Heavy Load (100 VUs, 1000 RPS)

| Metric | Value |
|--------|-------|
| Avg Latency | ~100ms |
| P95 Latency | ~250ms |
| P99 Latency | ~500ms |
| Throughput | ~1000 RPS |
| Error Rate | < 1% |

## Identifying Bottlenecks

### High Latency

If latency is consistently high:
1. Check CPU usage: `top` or `htop`
2. Check memory usage: `free -h`
3. Check network: `iftop` or `nethogs`
4. Profile with `perf`: `perf record -p <pid> -g -- sleep 30`

### High Error Rate

If error rate is high:
1. Check server logs: `docker logs <container>`
2. Verify authentication is configured correctly
3. Check rate limiting settings
4. Verify replay protection nonce generation

### Connection Timeouts

If connections are timing out:
1. Increase the number of file descriptors: `ulimit -n 65536`
2. Check TCP backlog: `sysctl net.core.somaxconn`
3. Increase connection pool size in k6 script

## Continuous Benchmarking

To run benchmarks regularly:

```bash
# Run daily at 2 AM
0 2 * * * cd /path/to/stellar-router && ./scripts/load-test.sh 5m 50 500 > /tmp/benchmark-$(date +\%Y\%m\%d).log 2>&1
```

## Docker Compose Load Testing

To run load tests against the Docker Compose stack:

```bash
# Start the stack
docker-compose up -d

# Run load test
export BASE_URL="http://localhost:3000"
./scripts/load-test.sh 30s 10 100

# Stop the stack
docker-compose down
```

## Advanced: Custom Load Test Scripts

You can create custom k6 scripts for specific scenarios. See the embedded script in `load-test.sh` for an example.

Key k6 features:
- **Stages** — Ramp up/down load gradually
- **Thresholds** — Define pass/fail criteria
- **Custom Metrics** — Track application-specific metrics
- **Checks** — Validate response properties
- **Groups** — Organize related requests

For more information, see [k6 documentation](https://k6.io/docs/).

## Troubleshooting

### k6 command not found

Install k6 using the instructions above.

### Connection refused

Ensure the metrics exporter is running:
```bash
curl http://localhost:9090/health
```

### Too many open files

Increase the file descriptor limit:
```bash
ulimit -n 65536
```

### Out of memory

Reduce the number of VUs or duration:
```bash
./scripts/load-test.sh 10s 5 50
```

## References

- [k6 Documentation](https://k6.io/docs/)
- [k6 HTTP API](https://k6.io/docs/javascript-api/k6-http/)
- [k6 Metrics](https://k6.io/docs/javascript-api/k6-metrics/)
- [Performance Testing Best Practices](https://k6.io/docs/testing-guides/load-testing/)
