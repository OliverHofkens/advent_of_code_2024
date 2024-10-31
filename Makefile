devsetup:
	brew install openocd qemu cmake ninja dfu-util ccache
	rustup toolchain install stable --component rust-src
	rustup target add riscv32imc-unknown-none-elf
	cargo install espup
	cargo install espflash
	cargo install ldproxy
	espup install
	. ~/export-esp.sh
