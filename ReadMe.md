# M*Player MQTT Rust Plugin 

This plugin for the M*Player allows hooking up the player with MQTT. It's written in Rust.

## MQTT topics

The general structure of the topics is:

    <prefix>/<scope>/<player name>/<command>

* The *prefix* can be freely configured but defaults to "MStarPlayer".
* The *scope* is either `control` to interact with a player or `monitor` to observe its activities.
* The *player name* is the name of the player it appears with on screen.
* The *command* depends on the *scope* and is either a command to the player or an activity of it.

These topics are published by the plugin:

| Topic                                     | When                                                  |
|-------------------------------------------|-------------------------------------------------------|
| `<prefix>/monitor/<player name>/playing`  | the named player started playback                     |
| `<prefix>/monitor/<player name>/stopped`  | the named player stopped playback                     |
| `<prefix>/monitor/<player name>/next`     | the named player moved to the next playlist entry     |
| `<prefix>/monitor/<player name>/previous` | the named player moved to the previous playlist entry |
| `<prefix>/monitor/<player name>/position` | the named player playback position changed            |

Most topics have no payload, except for the `position` command. It contains the string representation of the floating point value of the current playback position in seconds.
While playback is happening messages are published as often as the player informs the plugin about an updated playback position. This is usually multiple times per second.

These topics are being subscribed to by the plugin:

| Topic                                     | Will                                                  |
|-------------------------------------------|-------------------------------------------------------|
| `<prefix>/control/<player name>/play`     | start playback of the named player                    |
| `<prefix>/control/<player name>/stop`     | stop playback of the named player                     |
| `<prefix>/control/<player name>/next`     | move the named player to the next playlist entry      |
| `<prefix>/control/<player name>/previous` | move the named player to the previous playlist entry  |

The payload is ignored for all those topics.

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
