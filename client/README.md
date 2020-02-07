# client

The desktop application that communicates with the remote embedded device.

![Screenshot of client application](../img/client_1.png?raw=true "Client application")

## Assigning a device
In order to find whether the transceiver has been connected, the device's `vid` and `pid` are used. While the device is connected to the system, they can be found using one of the following commands:

* Linux: Use `lsusb` to display information about connected devices.
* Windows: Go to `Device Manager`, right click the device, click `Properties`, go to the `Details` tab and then select `Hardware IDs`.

The device `vid` and `pid` can then be assigned in `config/config.ron`.
