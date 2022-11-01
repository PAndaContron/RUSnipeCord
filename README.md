# RUSnipeCord

Rutgers course sniper which notifies you over a Discord webhook.

I made this because I wanted to get notified faster when a course I was sniping opened up, and email wasn't quite fast enough.

Note that this sniper doesn't use a Discord bot; instead, it uses a webhook, which only allows it to send messages.
An example configuration file is included in `config-ex.json`; the actual configuration should go in `config.json`.
Each of the fields in the config file is explained [here](src/dat.rs#L6).
Once it's configured, you can run it with `cargo run --release`.
You should get a message through the webhook to confirm that the sniper is running.
