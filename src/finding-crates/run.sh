#!/usr/bin/env bash

set -xe

cargo build --release

git clone --depth=1 https://github.com/rust-lang/crates.io-index.git

cp ./target/release/full_init /usr/local/bin/init-meili-crates
cp ./target/release/live /usr/local/bin/live-meili-crates

/usr/local/bin/init-meili-crates

echo please add the following line to your cronjobs with 'crontab -e'
echo and check that it is not already there ':)'
echo
echo "*/10 * * * * MEILI_HOST_URL=$MEILI_HOST_URL MEILI_INDEX_UID=$MEILI_INDEX_UID MEILI_API_KEY=$MEILI_API_KEY /usr/local/bin/live-meili-crates"
