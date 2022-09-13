RPI_VERSION ?= 4

BOOTMNT ?= ../firmware

ARMGNU ?= aarch64-linux-gnu

COPS = -DRPI_VERSION=$(RPI_VERSION) -Wall -nostdlib -nostartfiles -ffreestanding \
	   -Iinclude -mgeneral-regs-only

ASMOPS = -Iinclude

BUILD_DIR = build
SRC_DIR = src

all : armstub

clean :
	rm -rf $(BUILD_DIR) *.img

build/armstub_s.o: armstub.S
	mkdir -p $(@D)
	$(ARMGNU)-gcc $(COPS) -MMD -c $< -o $@

armstub: build/armstub_s.o
	$(ARMGNU)-ld --section-start=.text=0 -o build/armstub.elf build/armstub_s.o
	$(ARMGNU)-objcopy build/armstub.elf -O binary armstub-new.bin
	mv armstub-new.bin $(BOOTMNT)/
	sync