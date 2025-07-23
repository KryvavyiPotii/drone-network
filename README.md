# drone-network

An app that aims to model a UAV networks and the impact of electronic warfare and malware on them.
It is a reimagined and enhanced version of [drone-network-proto](https://github.com/KryvavyiPotii/drone-network-proto).

## Render image legend

* **Green circle** - command center transmission area.
* **Yellow circle** - destination point.
* **Orange circle** - transmission area of an attacker device that executes GPS spoofing attack.
* **Red circle** - transmission area of an electronic warfare device that suppresses GPS signal.
  On contact a drone loses its global position and moves in the same horizontal direction in which it moved before the contact.
* **Pink circle** - transmission area of an attacker device that spreads malware.
* **Blue circle** - transmission area of an electronic warfare device that suppresses control signal.

## Usage

```console
$ drone_network -h
Models drone networks.

Usage: drone_network-x64-linux [OPTIONS] --trx <trx system type> --topology <network topology>

Options:
  -x, --experiment <experiment title>
          Choose experiment title [possible values: custom, gpsewd, gpsspoof, malware,
move, signalloss]
      --im <network model path>
          Deserialize network model from `.json` file and use it
      --trx <trx system type>
          Choose device TRX system type [possible values: color, strength]
  -t, --topology <network topology>
          Choose network topology [possible values: mesh, star]
  -n <drone count>
          Set the number of drones in the network (non-negative integer) [default: 100]
  -d, --delay-multiplier <delay multiplier>
          Set signal transmission delay multiplier (non-negative float) [default: 0.0]
      --display-propagation
          Show malware propagation as well ("malware" experiment)
  -i, --infection <malware type>
          Choose infection type ("malware" experiment) [possible values: dos, indicator]
      --od <output directory path>
          Serialize network model data on each iteration to specified directory
      --no-plot
          Do not render a GIF plot
  -c, --caption <plot caption>
          Set the plot caption [default: ]
      --width <plot width>
          Set the plot width (in pixels) [default: 400]
      --height <plot height>
          Set the plot height (in pixels) [default: 300]
      --time <simulation time>
          Set the simulation time (in millis) [default: 15000]
  -h, --help
          Print help
  -V, --version
          Print version
```
