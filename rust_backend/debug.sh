cargo build -p ws_server
mv ./target/debug/libws_server.so ./modules/ws_server
cargo build -p simple_authenticator_application
mv ./target/debug/libsimple_authenticator_application.so ./modules/ws_server/assets/lib
cargo build -p chat_ws_server_application
mv ./target/debug/libchat_ws_server_application.so ./modules/ws_server/assets/lib
cargo build -p simple_authenticator
mv ./target/debug/libsimple_authenticator.so ./modules/simple_authenticator
cargo build -p db_connector
mv ./target/debug/libdb_connector.so ./modules/db_connector
cargo build -p chat_server
mv ./target/debug/libchat_server.so ./modules/chat_server
# rm -r ./target
cargo run -p main_backend
