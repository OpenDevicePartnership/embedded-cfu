# embedded-cfu
CFU protocol crate

This crate implements all commands and responses as structs per the Windows [CFU spec](https://learn.microsoft.com/en-us/windows-hardware/drivers/cfu/cfu-specification).

CfuComponent traits are included to cover behavior for:
- accessing necessary component info
- preparing, writing to, and finishing touch for the component's storage
- any post-update requirements

Client traits are included to cover behavior for:
- Preparing CFU components for updates
- Processing any CFU commands received from the Host
All EC-solutions which are intended to be updateable via CFU should implement the Receiver traits

Host traits are included to cover host behavior as per [Host programming command sequence](https://learn.microsoft.com/en-us/windows-hardware/drivers/cfu/cfu-specification#41-firmware-update-programming-command-sequence) for:
- Host states as it iterates through the offer list
- Updating content by breaking a CfuImage into properly sized chunks and sending them via cfu commands
Some EC's, such as those which will not be receiving the CFU offers and content from some OS driver for CFU, will need to implement the Host traits to be able to update components themselves.

Lastly, a CfuWriter trait is defined which is intended as bus-agnostic. It serves the dual purpose of communicating between the Host and Client as well as writing/reading to a component itself.
