devsetup:
	brew install openocd
	brew install qemu
	rustup toolchain install stable --component rust-src
	rustup target add riscv32imc-unknown-none-elf
	cargo install espup
	cargo install espflash
	espup install
	. ~/export-esp.sh
