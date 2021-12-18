# Generate Data

This folder contains generated fake data from [https://generatedata.com/generator](https://generatedata.com/generator).

This data is used to generate `../src/generated.rs` so that the WASM build is compiled with data to be
used in the web demo.

`generated.rs` can be rebuild via the following command:

``` sh
cd fake_data # make sure you're in this folder
python ./generate.py
```
