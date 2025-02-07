# embedded-cfu
CFU protocol crate

This crate implements all commands and responses as structs per the Windows [spec](https://learn.microsoft.com/en-us/windows-hardware/drivers/cfu/cfu-specification). Furthermore, it defines a trait for the Writer itself that is agnostic to the communication method

[](https://learn.microsoft.com/en-us/windows-hardware/drivers/cfu/cfu-firmware-implementation-guide)
