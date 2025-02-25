cargo build --release --target thumbv6m-none-eabi
elf2uf2-rs target/thumbv6m-none-eabi/release/simple_lcd target/thumbv6m-none-eabi/release/simple_lcd.uf2
cp target/thumbv6m-none-eabi/release/simple_lcd.uf2 /Volumes/RPI-RP2/simple_lcd.uf2