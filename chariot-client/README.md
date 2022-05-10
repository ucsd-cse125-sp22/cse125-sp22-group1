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

You can download the resources from [our shared drive](https://drive.google.com/drive/u/0/folders/0AK4EapywVEKbUk9PVA) directly.
Maybe you could use a tool like rclone too but I'm too dumb for that.

You can download to any folder and set that path in `resource_folder` in `config.yaml`,
but `./resources` is the default. So, you can expect models to be in `./resources/models`
and sounds to be in `./resources/sounds`, by convention.