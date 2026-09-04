# Legacy backend-only release launcher. The supported unified launcher is `../start.sh` on Linux.

cargo build -r -p ws_server
rm ./modules/ws_server/ws_server.dll
move ./target/release/ws_server.dll ./modules/ws_server/

cargo build -r -p simple_authenticator
rm ./modules/simple_authenticator/simple_authenticator.dll
move ./target/release/simple_authenticator.dll ./modules/simple_authenticator/

cargo build -r -p db_connector
rm ./modules/db_connector/db_connector.dll
move ./target/release/db_connector.dll ./modules/db_connector/

cargo build -r -p chat_server
rm ./modules/chat_server/chat_server.dll
move ./target/release/chat_server.dll ./modules/chat_server/

cargo build -r -p control_panel
rm ./modules/control_panel/control_panel.dll
move ./target/release/control_panel.dll ./modules/control_panel/

cargo run -r -p main_backend
