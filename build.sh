#!bin /bin/bash
wasm-pack build --target nodejs;
cd ./pkg && npm link;
cd ../node && npm link htmlentity && npm run test;