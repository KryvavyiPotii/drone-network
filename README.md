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

Usage: drone_network-x64-linux [OPTIONS] --slr <control signal loss response>

Options:
  -x <experiment title>
          Choose experiment title [possible values: custom, ewd, gpsspoof, malware, move, signalloss]
      --tx <tx module type>
          Choose TX system type [possible values: lvl, str]
      --slr <control signal loss response>
          Choose control signal loss response (except "signalloss" experiment) [possible values: ascend, ignore, hover, rth, shutdown]
      --topology <network topology>
          Choose network topology [possible values: mesh, star]
  -n <drone count>
          Set the number of drones in the network (non-negative integer) [default: 100]
      --dm <delay multiplier>
          Set signal transmission delay multiplier (non-negative float) [default: 0.0]
      --ewf <electronic warfare frequency>
          Choose EW frequency ("ewd" experiment) [possible values: control, gps]
      --ar <attacker device area radius>
          Set attacker device area radius (non-negative float) ("ewd", "gpsspoof" and "malware" experiments)
      --mt <malware type>
          Choose malware type ("malware" experiment) [possible values: dos, indicator]
      --ji <json input path>
          Deserialize network model from `.json` file and use it ("custom" experiment)
      --jo <json directory output path>
          Serialize network model data on each iteration to `.json` files in specified directory
      --time <simulation time>
          Set the simulation time (non-negative integer, in millis) [default: 15000]
      --no-plot
          Do not render a GIF plot
  -c, --caption <plot caption>
          Set the plot caption [default: ]
      --pw <plot width>
          Set the plot width (in pixels) [default: 400]
      --ph <plot height>
          Set the plot height (in pixels) [default: 300]
      --cp <camera pitch>
          Set camera pitch (in radians) [default: 0.15]
      --cy <camera yaw>
          Set camera yaw (in radians) [default: 0.5]
  -v, --verbose
          Show full log output
  -h, --help
          Print help
  -V, --version
          Print version
```
