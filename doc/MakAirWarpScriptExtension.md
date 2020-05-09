# Introduction

A WarpScript extension has been created specifically for working with the MakAir telemetry data. This extension adds a function to the WarpScript language for decoding the raw telemetry messages and convert them into WarpScript maps that can then easily be manipulated inside Warp 10.

# Installing the extension

At the moment the extension must be compiled manually by following those steps:

## Clone the git repository from GitHub

```
git clone https://github.com/senx/warp10-ext-makair.git
```

## Build the code

```
cd warp10-ext-makair
./gradlew shadowJar
```

## Deploy the extension into your Warp 10 instance

The `.jar` file resulting from the previous step must be copied in the `lib` directory of your Warp 10 instance:

```
cp build/libs/warp10-ext-makair.jar /opt/warp10/lib
```

A configuration file must then be added which will instruct Warp 10 to load the extension. Add the following lines to a file named `etc/conf.d/90-makair.conf`:

```
warpscript.extension.makair = io.warp10.ext.makair.MakAirWarpScriptExtension
```

You must then restart your Warp 10 instance. The log file `logs/warp10.log` should display a message similar to the one below:

```
2020-05-08T19:47:09,785 main INFO  script.WarpScriptLib - LOADED extension 'io.warp10.ext.makair.MakAirWarpScriptExtension'
```

# Using the extension

The extension adds a function named `MAKAIR.TELEMETRY->` to WarpScript. This function takes as input a byte array containing a raw telemetry message and outputs a map containing the various elements of the message.

The following example shows how it is used:

```
'TzoBA2RldgAtAB9BTlAHIDUyRwkAAAAAF2Yg2Qo=' B64-> MAKAIR.TELEMETRY->
```

You can experiment with a live example using this [WarpScript Snapshot](https://snapshot.senx.io/0005a52877d7ed8d-0-2-2fffd5880ef67922).

With this function, raw frames can be stored in Warp 10 as they are produced and later decoded in WarpScript.

Storing raw frames can be done easily by formatting the input to the `/api/v0/update` endpoint like below:

```
<TIMESTAMP>// makair.telemetry.raw{device=XXX} b64:TzoBA2RldgAtAB9BTlAHIDUyRwkAAAAAF2Yg2Qo=
```

Where `<TIMESTAMP>` is the timestamp when the frame was received, expressed in your platform time units.