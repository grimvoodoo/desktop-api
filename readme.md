# Desktop API

The purpose of this app is to create an API that I can call from home assist to play/pause media on my pc. I might add more to it but if anyone else has the same use-case you can use this to do that.

## How to build

You will need rust installed, you can find instructions on how to do that [here](https://www.rust-lang.org/tools/install)

Once that is installed you can run these commands to build it:

```bash
git clone https://github.com/grimvoodoo/desktop-api.github
cd desktop-api
cargo build --release
```
That will take a minuite to compile and then you will find the binary in `target/release/smart-speaker`

You can execute that to run the service. It will listen for incoming connections on port 3000. Call the endpoint you want to access to trigger the action.

## To do

I want to add some auth to this api, as its on the local network I am not too concerned about someone sneaking into my wifi and pausing my media but it would be good to add anyway.

