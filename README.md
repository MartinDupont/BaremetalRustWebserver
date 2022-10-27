# Bare metal webserver

This repo contains code for an operating system for the Raspberry Pi 4 which is to be a minimal web server. This is 
mostly intended as a hobby project for learning Rust and low-level development.

The code is based on the course material for Georgia Tech CS3210 "Design of Operating Systems", available [here](https://tc.gts3.org/cs3210/2020/spring/index.html),
and is forked from [this](https://github.com/sslab-gatech/cs3210-rustos-public) github repo. This course was in turn based on [CS140e: An Experimental Course on Operating Systems](https://cs140e.sergio.bz/)
by [Sergio Benitez](https://sergio.bz/).

The code has however been ported to work on the Raspberry Pi4. 

The operating system supports:
- Multithreading and Multicore
- FAT32 file system
- Dynamic memory allocation
- User space programs

The ethernet driver is still a work in progress due to the significant differences between the ethernet implementations
for the Raspberry Pi 3 and 4. 

## Installing
Running `/bin/setup.sh` on a linux machine (works on Ubuntu 22.04) should install all the necessary packages and toolchains etc.
To build the kernel, `cd` into `/kern` and run `make`, which should build the OS. Then, insert the SD card from the raspberry
pi into the computer and run `python bin/intsall-kernel.py`. This will flash the OS and all the necessary firmware onto the SD
card.

## Running.
The raspberry Pi communicates with the host PC using USB connected to the UART in the Raspberry Pi. This
can be accomplished by connecting a USB module to GPIO pins 14 and 15. See the course notes for more details. 

Once the Pi is connected and plugged in via USB, you can communicate with it by running `sudo screen /dev/ttyUSB0 921600`.
