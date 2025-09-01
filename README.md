# ChainPipe
Dataflow-driven filtering and on-chain event listening.

```bash
cd chain-pipe/

cargo run --bin uniswap-source pair-created-event \
  --ws-url wss://reth-ethereum.ithaca.xyz/ws \
  --factory-address 0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f \
  --server-url nats-server:4222 \
  --subject-name eth.univ2.factory.pair_created.0

cargo run --bin uniswap-source sync-event \
  --ws-url wss://reth-ethereum.ithaca.xyz/ws \
  --server-url nats-server:4222 \
  --subject-name eth.univ2.pair.sync.0

cargo run --bin pair-enricher -- \
  --http-url https://reth-ethereum.ithaca.xyz/rpc \
  --server-url nats-server:4222 \
  --subject-input eth.univ2.factory.pair_created.0 \
  --stream-name ETH_UNIV2_FACTORY \
  --kv-bucket univ2_new_pairs \
  --pair-address 0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc \
  --pair-address 0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852

  cargo run --bin price-injector -- \
  --server-url nats-server:4222 \
  --subject-input eth.univ2.pair.sync.0 \
  --subject-output eth.univ2.pair.sync.1 \
  --stream-name ETH_UNIV2_PAIR \
  --kv-bucket univ2_new_pairs 

  cargo run --bin price-sink -- \
  --server-url nats-server:4222 \
  --subject-name eth.univ2.pair.sync.1 \
  --stream-name ETH_UNIV2_PAIR \
  --dsn postgres://postgres:password@localhost:5432/prices

```

```bash
nats stream add -h

nats --server=nats-server:4222 stream rm ETH_UNIV2_EVENTS -f

nats --server=nats-server:4222 stream add ETH_UNIV2_FACTORY --subjects="eth.univ2.factory.>" \
  --storage=file \
  --defaults    

nats --server=nats-server:4222 stream add ETH_UNIV2_PAIR --subjects="eth.univ2.pair.>" \
  --storage=file \
  --defaults    

nats --server=nats-server:4222 stream ls
nats --server=nats-server:4222 stream info ETH_UNIV2_PAIR
nats --server=nats-server:4222 sub eth.univ2.factory.pair_created.0
nats --server=nats-server:4222 sub eth.univ2.pair.sync.0
nats --server=nats-server:4222 sub eth.univ2.pair.sync.1

nats consumer add -h
nats --server=nats-server:4222 consumer add ETH_UNIV2_PAIR consumer-test \
--defaults
nats --server=nats-server:4222 consumer ls ETH_UNIV2_FACTORY
nats --server=nats-server:4222 consumer rm ETH_UNIV2_PAIR N0eflR28

nats --server=nats-server:4222 \
  consumer next ETH_UNIV2_PAIR consumer-test --count=10

nats --server=nats-server:4222 account info  
nats --server=nats-server:4222 kv rm univ2_new_pairs
nats --server=nats-server:4222 kv ls
nats --server=nats-server:4222 kv get --raw univ2_new_pairs 0x538e4c324a97ccd381383b3ac6200cd3a47f6ed9
nats --server=nats-server:4222 kv history univ2_new_pairs 0x48cf2c7c0e3c90793a1a3459cb49720da1a10071 

```
