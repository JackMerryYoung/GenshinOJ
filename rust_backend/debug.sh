cargo build -p ws_server
mv ./target/debug/libws_server.so ./modules/ws_server
cargo build -p simple_authenticator
mv ./target/debug/libsimple_authenticator.so ./modules/simple_authenticator
cargo build -p db_connector
mv ./target/debug/libdb_connector.so ./modules/db_connector
cargo build -p chat_server
mv ./target/debug/libchat_server.so ./modules/chat_server
cargo build -p judge
mv ./target/debug/libjudge.so ./modules/judge
cargo build -p userish
mv ./target/debug/libuserish.so ./modules/userish
# control_panel: debug build bypasses the admin token (dev backdoor). Use build&run.sh for release.
cargo build -p control_panel
mv ./target/debug/libcontrol_panel.so ./modules/control_panel
# rm -r ./target
cargo run -p main_backend
