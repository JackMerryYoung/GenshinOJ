cargo build -r -p ws_server
mv ./target/release/libws_server.so ./modules/ws_server
# rm -r ./target
cargo run -r -p main_backend
