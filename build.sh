cargo build --release --target thumbv6m-none-eabi
elf2uf2-rs target/thumbv6m-none-eabi/release/lilbud target/thumbv6m-none-eabi/release/lilbud.uf2
cp target/thumbv6m-none-eabi/release/lilbud.uf2 /Volumes/RPI-RP2/lilbud.uf2