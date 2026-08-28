# M*Player MQTT Rust Plugin 

This plugin for the M*Player allows hooking up the player with MQTT. It's written in Rust.

## MQTT topics

The general structure of the topics is:

    <prefix>/<scope>/<player name>/<command>

* The *prefix* can be freely configured but defaults to "MStarPlayer".
* The *scope* is either `control` to interact with a player or `monitor` to observe its activities.
* The *player name* is the name of the player it appears with on screen.
* The *command* depends on the *scope* and is either a command to the player or an activity of it.


### Publishing

For each player the plugin will published to these topics under `<prefix>/monitor/<player name>/<command>`:

| Command    | When                              | Payload                             |
|------------|-----------------------------------|-------------------------------------|
| `playing`  | playback started                  | *none*                              |
| `stopped`  | playback stopped                  | *none*                              |
| `next`     | jumped to next playlist entry     | *none*                              |
| `previous` | jumped to previous playlist entry | *none*                              |
| `position` | playback position changed         | floating point in seconds as string |

While playback is happening messages are published as often as the player informs the plugin about an updated playback position. This is usually multiple times per second.

### Subscribed

For each player the plugin subscribes to these topics under `<prefix>/control/<player name>/<command>`:

| Command    | Will                                | Payload       |
|------------|-------------------------------------|---------------|
| `play`     | start playback                      | *none*        |
| `stop`     | stop playback                       | *none*        |
| `next`     | jump to the next playlist entry     | *none*        |
| `previous` | jump to the previous playlist entry | *none*        |

## Building

The plugin can be built using:

    cargo build

It's also possible to cross-compile it for Windows:

    rustup target add x86_64-pc-windows-gnu
    SLINT_NO_QT= cargo build --target x86_64-pc-windows-gnu

When cross-compiling Qt might not be available so `SLINT_NO_QT` is set [as documented](https://github.com/slint-ui/slint/blob/0b2e95f3115ba0f28256acebeb393271bb81d9a8/docs/install_qt.md#how-to-disable-the-qt-backend).

## Testing

Some tests can be run using:

    cargo test

The more extensive tests require an MQTT broker on `127.0.0.1:1883`. Then they can be run using:

    cargo test -- --include-ignored

## License

ISC
