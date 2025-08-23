# ChainPipe
Dataflow-driven filtering and on-chain event listening.

```bash
cd chain-pipe/

cargo run --bin source-uniswap -- \
  --ws-url wss://reth-ethereum.ithaca.xyz/ws \
  --factory-address 0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f \
  --broker-url sc:9003 \
  --topic-name univ2-factoty-test

cargo run --bin enrich-pair -- \
  --http-url https://reth-ethereum.ithaca.xyz/rpc \
  --server-url nats-server:4222 \
  --subject-input eth.univ2.pair_created.0 \
  --stream-name ETH_UNIV2_EVENTS \
  --kv-bucket univ2_new_pairs

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

nats --server=nats-server:4222 account info  
nats --server=nats-server:4222 kv get --raw univ2_new_pairs 0x538e4c324a97ccd381383b3ac6200cd3a47f6ed9
nats --server=nats-server:4222 kv history univ2_new_pairs 0xc952cd23b0c053edb74a8e4ee2f7d254bcefe158

```
