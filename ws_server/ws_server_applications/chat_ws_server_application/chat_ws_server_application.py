import sys, json, enum, warnings

import websockets

from ... import ws_server


class chat_ws_server_application_log_level(enum.Enum):
    LEVEL_INFO = 0
    LEVEL_DEBUG = 1
    LEVEL_WARNING = 2
    LEVEL_ERROR = 3


class chat_ws_server_application(ws_server.ws_server_application_protocol):

    def log(
        self,
        log: str,
        log_level: chat_ws_server_application_log_level = chat_ws_server_application_log_level.LEVEL_INFO,
    ):
        call_frame = sys._getframe(1)
        if log_level is chat_ws_server_application_log_level.LEVEL_INFO:
            print(
                "\033[1;2m[WS_SERVER] [CHAT_WS_SERVER_APP] [INFO] {} (file `{}`, function `{}` on line {})\033[0m".format(
                    log,
                    call_frame.f_code.co_filename,
                    call_frame.f_code.co_name,
                    call_frame.f_lineno,
                )
            )
        if log_level is chat_ws_server_application_log_level.LEVEL_DEBUG:
            print(
                "\033[1;34m[WS_SERVER] [CHAT_WS_SERVER_APP] [DEBUG] {} (file `{}`, function `{}` on line {})\033[0m".format(
                    log,
                    call_frame.f_code.co_filename,
                    call_frame.f_code.co_name,
                    call_frame.f_lineno,
                )
            )
        if log_level is chat_ws_server_application_log_level.LEVEL_WARNING:
            print(
                "\033[1;33m[WS_SERVER] [CHAT_WS_SERVER_APP] [WARNING] {} (file `{}`, function `{}` on line {})\033[0m".format(
                    log,
                    call_frame.f_code.co_filename,
                    call_frame.f_code.co_name,
                    call_frame.f_lineno,
                )
            )
        if log_level is chat_ws_server_application_log_level.LEVEL_ERROR:
            print(
                "\033[1;31m[WS_SERVER] [CHAT_WS_SERVER_APP] [ERROR] {} (file `{}`, function `{}` on line {})\033[0m".format(
                    log,
                    call_frame.f_code.co_filename,
                    call_frame.f_code.co_name,
                    call_frame.f_lineno,
                )
            )

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
            "chat_server"
        ).message_box[content["username"]] = dict()
        self.ws_server_instance.server_instance.get_module_instance(
            "chat_server"
        ).message_box[content["username"]]["message_queue"] = []
        self.ws_server_instance.server_instance.get_module_instance(
            "chat_server"
        ).message_box[content["username"]]["websocket_protocol"] = websocket_protocol

    async def on_close_connection(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict = None
    ):
        await super().on_close_connection(websocket_protocol, content)

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

        try:
            del self.ws_server_instance.server_instance.get_module_instance(
                "chat_server"
            ).message_box[content["username"]]
        except KeyError:
            pass

    def get_md5(self, data):
        import hashlib

        hash = hashlib.md5("add-some-salt".encode("utf-8"))
        hash.update(data.encode("utf-8"))
        return hash.hexdigest()

    async def on_chat_short(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict,
    ):
        warnings.warn(
            "on_chat_short is deprecated and will be removed in 1.2-mainstream."
        )
        try:
            if (
                content["from"]
                == self.ws_server_instance.server_instance.get_module_instance(
                    "ws_server"
                ).sessions[content["session_token"]]
            ):
                try:
                    self.ws_server_instance.server_instance.get_module_instance(
                        "chat_server"
                    ).message_box[content["to"]]["message_queue"].append(
                        {"from": content["from"], "messages": content["messages"]}
                    )
                    self.log(
                        "The user {} tried to use session token: {} to send message.".format(
                            content["from"], content["session_token"]
                        )
                    )
                except KeyError:
                    self.log(
                        "The user {} tried to send a message to whom is not online.".format(
                            content["from"]
                        )
                    )
            else:
                self.log(
                    "The user {} tried to use a fake session token.".format(
                        content["from"]
                    )
                )
        except KeyError:
            self.log(
                "The user {} tried to use a fake session token.".format(content["from"])
            )

    async def on_chat_user(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict,
    ):
        try:
            if (
                content["from"]
                == self.ws_server_instance.server_instance.get_module_instance(
                    "ws_server"
                ).sessions[content["session_token"]]
            ):
                try:
                    print(
                        list(
                            self.ws_server_instance.server_instance.get_module_instance(
                                "chat_server"
                            ).message_box.keys()
                        )
                    )
                    self.ws_server_instance.server_instance.get_module_instance(
                        "chat_server"
                    ).message_box[content["to"]]["message_queue"].append(
                        {"from": content["from"], "messages": content["messages"]}
                    )
                    self.log(
                        "The user {} tried to use session token: {} to send message.".format(
                            content["from"], content["session_token"]
                        )
                    )

                    try:
                        response = {
                            "type": "chat_echo",
                            "content": {
                                "status": 1,
                                "messages": content["messages"],
                            },
                        }
                        await websocket_protocol.send(json.dumps(response))
                    except websockets.exceptions.ConnectionClosed:
                        pass

                except KeyError:
                    self.log(
                        "The user {} tried to send a message to whom is not online.".format(
                            content["from"]
                        )
                    )
                    try:
                        response = {
                            "type": "chat_echo",
                            "content": {
                                "status": 0,
                                "reason": "offline",
                            },
                        }
                        await websocket_protocol.send(json.dumps(response))
                    except websockets.exceptions.ConnectionClosed:
                        pass
            else:
                self.log(
                    "The user {} tried to use a fake session token.".format(
                        content["from"]
                    )
                )
                try:
                    response = {
                        "type": "chat_echo",
                        "content": {
                            "status": 0,
                            "reason": "fake_token",
                        },
                    }
                    await websocket_protocol.send(json.dumps(response))
                except websockets.exceptions.ConnectionClosed:
                    pass
        except KeyError:
            self.log(
                "The user {} tried to use a fake session token.".format(content["from"])
            )
