# Build Websocket Server
if [ ! -e "./modules" ]; then
    mkdir ./modules
fi

if [ ! -e "./modules/ws_server" ]; then
    mkdir ./modules/ws_server
fi

cargo build -r -p ws_server > /dev/null
if [ -e "./target/release/libws_server.so" ]; then
    if [ -e "./modules/ws_server/libws_server.so" ]; then
        if [ "$(cat ./target/release/libws_server.so)" != "$(cat ./modules/ws_server/libws_server.so)" ]; then
            mv ./target/release/libws_server.so ./modules/ws_server
        fi
    fi
fi

###############################################################

# Building Websocket Application Libraries
if [ ! -e "./modules/ws_server/assets" ]; then
    mkdir ./modules/ws_server/assets
fi

###############################################################

# Building Simple Authenticator
if [ ! -e "./modules/simple_authenticator" ]; then
    mkdir ./modules/simple_authenticator
fi

cargo build -r -p simple_authenticator
if [ -e "./target/release/libsimple_authenticator.so" ]; then
    if [ -e "./modules/simple_authenticator/libsimple_authenticator.so" ]; then
        if [ "$(cat ./target/release/libsimple_authenticator.so)" != "$(cat ./modules/simple_authenticator/libsimple_authenticator.so)" ]; then
            echo "Moving simple_authenticator..."
            mv ./target/release/libsimple_authenticator.so ./modules/simple_authenticator
        else
            echo "No need to move simple_authenticator."
        fi
    else
        echo "Moving simple_authenticator..."
        mv ./target/release/libsimple_authenticator.so ./modules/simple_authenticator
    fi
fi

###############################################################

# Building Database Connector
if [ ! -e "./modules/db_connector" ]; then
    mkdir ./modules/db_connector
fi

cargo build -r -p db_connector
if [ -e "./target/release/libdb_connector.so" ]; then
    if [ -e "./modules/db_connector/libdb_connector.so" ]; then
        if [ "$(cat ./target/release/libdb_connector.so)" != "$(cat ./modules/db_connector/libdb_connector.so)" ]; then
            echo "Moving db_connector..."
            mv ./target/release/libdb_connector.so ./modules/db_connector
        else
            echo "No need to move db_connector."
        fi
    else
        echo "Moving db_connector..."
        mv ./target/release/libdb_connector.so ./modules/db_connector
    fi
fi

###############################################################

# Building Chat Server
if [ ! -e "./modules/chat_server" ]; then
    mkdir ./modules/chat_server
fi

cargo build -r -p chat_server
if [ -e "./target/release/libchat_server.so" ]; then
    if [ -e "./modules/chat_server/libchat_server.so" ]; then
        if [ "$(cat ./target/release/libchat_server.so)" != "$(cat ./modules/chat_server/libchat_server.so)" ]; then
            echo "Moving chat_server..."
            mv ./target/release/libchat_server.so ./modules/chat_server
        else
            echo "No need to move chat_server."
        fi
    else
        echo "Moving chat_server..."
        mv ./target/release/libchat_server.so ./modules/chat_server
    fi  
fi

###############################################################

# Building Main Backend
cargo run -r -p main_backend

###############################################################