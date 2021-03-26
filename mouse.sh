#!/bin/bash

device='/dev/input/mice'

sudo systemctl stop udevmon.service
sleep 1
echo start

sudo intercept -g "$device" | sudo nice -n -20 target/debug/key-mods-rs | sudo uinput -d "$device" -c /home/shiro/project/key-mods-rs/keyboard.yaml &

proc_pid=$?

sleep 10

echo stop

sudo killall -9 intercept

sleep 1

sudo systemctl restart udevmon.service
echo restarted udevmon
