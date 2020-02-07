# embedded

## Setup

Make sure to run to initiate the connection with the embedded device:

```shell
openocd -f interface/stlink-v2-1.cfg -f target/stm32f3x.cfg
```

While in the `embedded` directory, the embedded device's generated log can be viewed by running:
```shell
itmdump -F -f itm.txt
```

To upload new code, while in the `embedded/` directory, run:

```
cargo run
```

