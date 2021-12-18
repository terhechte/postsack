# Postsack Web

This is the WASM / Web version of Postsack. It uses fake email data to provide a web demo
so that interested parties can try out Postsack native / the app without having to install
it on their device.

## Building Postsack Web

First, you need to make sure all dependencies are installed:

``` sh
cd postsack-web
./setup_web.sh
```

Once this is done, building can be performed with a single script:

``` sh
./build_web.sh
```

## Testing

In order to simplify testing, `build_web.sh` will launch a browser on `localhost:8080`.
By default, `setup_web.sh` will install the `basic-http-server` so that you can run it
in the `web_demo` folder prior to running `build-web.sh`:

``` sh
cd web_demo
basic-http-server -a 127.0.0.1:8080 .
```
