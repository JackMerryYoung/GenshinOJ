cargo build -p ws_server
mv ./target/debug/libws_server.so ./modules/ws_server
cargo build -p simple_ws_server_application
mv ./target/debug/libsimple_ws_server_application.so ./modules/ws_server/assets/lib
# rm -r ./target
cargo run -p main_backend
