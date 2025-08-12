```bash
# ADDRESS sc:9003
fluvio profile list

# PUBLIC spu:9010
fluvio cluster spu list

fluvio topic list
fluvio topic create quickstart-topic

fluvio produce quickstart-topic
fluvio consume quickstart-topic -B -d

nc -v sc 9003

fluvio consume uniswap-v2-factoty-event -B -d
```