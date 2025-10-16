# Dice Tumbler

Tumbling 3D dice using `devicemotion` accelerometer.

Live demo: https://rectalogic.com/tumbler/

## Development

devicemotion requires TLS

```sh-session
$ mkcert $(hostname)
```
Install `$(mkcert -CAROOT)/rootCA.pem` on device.
For iOS, see `VPN & Device Management` in Settings,
then enable trust in `Certificate Trust Settings`.

```sh-session
$ cargo install wasm-pack
$ wasm-pack build --target web
$ uv run --python 3.14 python -m http.server --protocol HTTP/1.1 --tls-key $(hostname)-key.pem --tls-cert $(hostname).pem

# Open https://<hostname>:8000 on device
```
