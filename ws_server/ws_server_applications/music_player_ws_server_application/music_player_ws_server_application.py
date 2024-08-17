import logging, websockets

from ... import ws_server


class music_player_ws_server_application(ws_server.ws_server_application_protocol):

    def log(
        self,
        log: str,
        log_level: ws_server.simple_ws_server_application_log_level = ws_server.simple_ws_server_application_log_level.LEVEL_INFO,
    ):
        if log_level is ws_server.simple_ws_server_application_log_level.LEVEL_INFO:
            print("[MUSIC_PLAYER_SERVER] [INFO] {}".format(log))
        if log_level is ws_server.simple_ws_server_application_log_level.LEVEL_DEBUG:
            print("[MUSIC_PLAYER_SERVER] [DEBUG] {}".format(log))
        if log_level is ws_server.simple_ws_server_application_log_level.LEVEL_WARNING:
            print("[MUSIC_PLAYER_SERVER] [WARNING] {}".format(log))
        if log_level is ws_server.simple_ws_server_application_log_level.LEVEL_ERROR:
            print("[MUSIC_PLAYER_SERVER] [ERROR] {}".format(log))

    async def on_login(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict,
    ):
        self.log(
            "The user {} tries to login with the hash: {}".format(
                content["username"], self.get_md5(content["password"])
            )
        )
        self.ws_server_instance.server_instance.get_module_instance(
            "music_player_server"
        ).playing_users[content["username"]] = {
            "username": content["username"],
            "already_playing": False,
            "websocket_protocol": websocket_protocol,
        }

    async def on_close_connection(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict = None
    ):
        await super().on_close_connection(websocket_protocol)

    async def on_quit(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict,
    ):
        self.log(
            "The user {} quitted with session token: {}".format(
                content["username"], content["session_token"]
            )
        )
        del self.ws_server_instance.server_instance.get_module_instance(
            "music_player_server"
        ).playing_users[content["username"]]

    def get_md5(self, data):
        import hashlib

        hash = hashlib.md5("add-some-salt".encode("utf-8"))
        hash.update(data.encode("utf-8"))
        return hash.hexdigest()

    async def on_music_list(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict,
    ):
        pass

    async def on_music_play(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict,
    ):
        pass

    async def on_music_quit(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict,
    ):
        pass
