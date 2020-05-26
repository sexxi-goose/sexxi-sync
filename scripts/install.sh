#!/usr/bin/env bash

SYSTEMD_DIR=/home/sexxi-goose/.config/systemd/user
BOT_DIR=/home/sexxi-goose/.bot

mkdir -p $SYSTEMD_DIR

rm -f $SYSTEMD_DIR/sexxi*

ln -s $PWD/systemd/sexxi-sync.service $SYSTEMD_DIR
ln -s $PWD/systemd/sexxi-sync.timer $SYSTEMD_DIR

rm -rf $BOT_DIR/release
cargo build --release --target-dir $BOT_DIR/bin
