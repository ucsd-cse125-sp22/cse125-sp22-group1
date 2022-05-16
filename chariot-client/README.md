# chariot-client

Graphics engine, input controller, and networking interface for chariot.

### running the game
```bash
cd chariot-client
# make source changes...
cargo run
```

### syncronizing the resources

We've been keeping the resources on GDrive because our half gig race track models
made LFS upset.

You can download the resources from [our shared drive](https://drive.google.com/drive/folders/15lMRdrPnF4swE1D9H9jeMP6LyWI5xRbT?usp=sharing) directly.
Maybe you could use a tool like rclone too but I'm too dumb for that.

You can download to any folder and set that path in `resource_folder` in `config.yaml`,
but `./resources` is the default. By convention,

* models are in `./resource/models`
* sounds are in `./resource/sounds`