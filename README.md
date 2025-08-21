# ChainPipe
Dataflow-driven filtering and on-chain event listening.

```bash
cd chain-pipe/

cargo run --bin source-uniswap -- \
  --ws-url wss://reth-ethereum.ithaca.xyz/ws \
  --factory-address 0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f \
  --broker-url sc:9003 \
  --topic-name univ2-factoty-test
```

```bash
nats stream add -h
nats --server=nats-server:4222 stream add ETH_UNIV2_EVENTS --subjects="eth.univ2.>" \
  --storage=file \
  --defaults    

nats --server=nats-server:4222 stream ls
nats --server=nats-server:4222 stream info ETH_UNIV2_EVENTS
nats --server=nats-server:4222 sub eth.univ2.pair_created.0

nats consumer add -h
nats --server=nats-server:4222 consumer add ETH_UNIV2_EVENTS consumer-test \
--defaults

nats --server=nats-server:4222 \
  consumer next ETH_UNIV2_EVENTS consumer-test --count=10

```
