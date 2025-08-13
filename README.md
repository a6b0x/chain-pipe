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