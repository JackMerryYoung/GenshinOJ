cargo build -r -p ws_server
mv ./target/release/libws_server.so ./modules/ws_server
cargo build -r -p simple_ws_server_application
mv ./target/release/libsimple_ws_server_application.so ./modules/ws_server/assets/lib
cargo build -r -p chat_ws_server_application
mv ./target/release/libchat_ws_server_application.so ./modules/ws_server/assets/lib
# rm -r ./target
cargo run -r -p main_backend
