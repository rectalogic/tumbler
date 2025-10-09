```sh-session
$ cargo install wasm-pack
$ wasm-pack build --target web --features web
$ python3 -m http.server
$ ngrok http 8000  # devicemotion requires TLS, so access via ngrok URL
```
