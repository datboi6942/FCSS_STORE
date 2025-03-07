#!/bin/bash

# Start the wallet RPC service
./monero-wallet-rpc \
  --wallet-file=store_wallet \
  --rpc-bind-port=18082 \
  --rpc-bind-ip=127.0.0.1 \
  --daemon-address=http://node.sethforprivacy.com:18089 \
  --log-file=monero-wallet-rpc.log \
  --log-level=2 \
  --max-concurrency=1 \
  --prompt-for-password
