import sys, enum, json, random

import websockets

from ... import ws_server


def generate_session_token(session_token_seed: int) -> str:
    if session_token_seed > 1:
        generated_session_token = ""
        generated_session_token = (
            generated_session_token
            + chr(session_token_seed * 1 % 26 + ord("a"))
            + chr(session_token_seed * 3 % 26 + ord("a"))
            + chr(session_token_seed * 5 % 26 + ord("a"))
            + chr(session_token_seed * 7 % 26 + ord("a"))
            + chr(session_token_seed * 9 % 26 + ord("a"))
            + chr(session_token_seed * 11 % 26 + ord("a"))
            + chr(session_token_seed * 13 % 26 + ord("a"))
            + chr(session_token_seed * 15 % 26 + ord("a"))
        )
        return generated_session_token + generate_session_token(
            int(session_token_seed / 5)
        )
    else:
        return "s"


class simple_ws_server_application_log_level(enum.Enum):
    LEVEL_INFO = 0
    LEVEL_DEBUG = 1
    LEVEL_WARNING = 2
    LEVEL_ERROR = 3


class simple_ws_server_application(ws_server.ws_server_application_protocol):
    """
    A simple, official implementation of websocket server application, providing some simple plugins.
    """

    def __init__(self, ws_server_instance: ws_server) -> None:
        super().__init__(ws_server_instance)
        self.ws_server_instance.server_instance.get_module_instance(
            "db_connector"
        ).database_cursor.execute(
            """
        CREATE TABLE IF NOT EXISTS users (
            id INT AUTO_INCREMENT PRIMARY KEY NOT NULL,
            username VARCHAR(256) NOT NULL,
            password VARCHAR(256) NOT NULL
        )
        """
        )
        self.ws_server_instance.server_instance.get_module_instance(
            "db_connector"
        ).database.commit()

    def log(
        self,
        log: str,
        log_level: simple_ws_server_application_log_level = simple_ws_server_application_log_level.LEVEL_INFO,
    ):
        """
        Logging method
        Args:
            log (str): Log information
            log_level (simple_ws_server_application_log_level): Logging level
        Info:
            It is suggested that you should override this method to distinguish between the official application and yours.
        """
        call_frame = sys._getframe(1)
        if log_level is simple_ws_server_application_log_level.LEVEL_INFO:
            print(
                "\033[1;2m[WS_SERVER] [SIMPLE_WS_SERVER_APP] [INFO] {} (file `{}`, function `{}` on line {})\033[0m".format(
                    log,
                    call_frame.f_code.co_filename,
                    call_frame.f_code.co_name,
                    call_frame.f_lineno,
                )
            )
        if log_level is simple_ws_server_application_log_level.LEVEL_DEBUG:
            print(
                "\033[1;34m[WS_SERVER] [SIMPLE_WS_SERVER_APP] [DEBUG] {} (file `{}`, function `{}` on line {})\033[0m".format(
                    log,
                    call_frame.f_code.co_filename,
                    call_frame.f_code.co_name,
                    call_frame.f_lineno,
                )
            )
        if log_level is simple_ws_server_application_log_level.LEVEL_WARNING:
            print(
                "\033[1;33m[WS_SERVER] [SIMPLE_WS_SERVER_APP] [WARNING] {} (file `{}`, function `{}` on line {})\033[0m".format(
                    log,
                    call_frame.f_code.co_filename,
                    call_frame.f_code.co_name,
                    call_frame.f_lineno,
                )
            )
        if log_level is simple_ws_server_application_log_level.LEVEL_ERROR:
            print(
                "\033[1;31m[WS_SERVER] [SIMPLE_WS_SERVER_APP] [ERROR] {} (file `{}`, function `{}` on line {})\033[0m".format(
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
        """
        Official callback method for login with a authentication plugin.
        """

        password_hash = self.get_md5(content["password"])
        self.log(
            "The user {} try to login with the hash: {}.".format(
                content["username"], password_hash
            )
        )
        self.ws_server_instance.server_instance.get_module_instance(
            "db_connector"
        ).database_cursor.execute(
            'SELECT password FROM users WHERE username = "{}";'.format(
                content["username"]
            )
        )
        tmp = self.ws_server_instance.server_instance.get_module_instance(
            "db_connector"
        ).database_cursor.fetchone()

        real_password_hash = None
        try:
            real_password_hash = tmp[0]
        except:
            self.log(
                "The user {} failed to login.".format(content["username"]),
                simple_ws_server_application_log_level.LEVEL_WARNING,
            )
            response = {
                "type": "quit",
                "content": {
                    "reason": "authentication_failure",
                    "request_key": content["request_key"],
                },
            }
            await websocket_protocol.send(json.dumps(response))
            response.clear()
            return

        if real_password_hash != None and real_password_hash == password_hash:
            new_session_token = generate_session_token(
                random.randint(1000000000000000, 10000000000000000)
            )

            self.log("The user {} logged in successfully.".format(content["username"]))
            if getattr(websocket_protocol, "is_logged_in", False) == True:
                for k, v in list(self.ws_server_instance.sessions.items()):
                    if v == content["username"]:
                        del self.ws_server_instance.sessions[k]
                
                self.log("Cleared the previous session token of {}.".format(content["username"]))

            self.ws_server_instance.sessions[new_session_token] = content["username"]
            self.log("The session token: {}".format(new_session_token))
            setattr(self, "username", content["username"])
            setattr(self, "session_token", new_session_token)
            setattr(websocket_protocol, "is_logged_in", True)
            response = {
                "type": "session_token",
                "content": {
                    "session_token": new_session_token,
                    "request_key": content["request_key"],
                },
            }
            await websocket_protocol.send(json.dumps(response))
            response.clear()
        else:
            self.log(
                "The user {} failed to login.".format(content["username"]),
                simple_ws_server_application_log_level.LEVEL_WARNING,
            )
            response = {
                "type": "quit",
                "content": {
                    "reason": "authentication_failure",
                    "request_key": content["request_key"],
                },
            }
            await websocket_protocol.send(json.dumps(response))
            response.clear()

    async def on_close_connection(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict = None, # Ignored one
    ):
        await super().on_close_connection(websocket_protocol, content)
        if getattr(websocket_protocol, "is_logged_in", False):
            setattr(websocket_protocol, "is_logged_in", False)
            await self.on_quit(
                websocket_protocol,
                {"username": self.username, "session_token": self.session_token},
            )
        else:
            await self.on_quit(websocket_protocol, None)

    async def on_quit(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict | None,
    ):
        if content != None:
            self.log(
                "The user {} quitted with session token: {}".format(
                    content["username"], content["session_token"]
                )
            )
            try:
                if (
                    content["username"]
                    == self.ws_server_instance.server_instance.get_module_instance(
                        "ws_server"
                    ).sessions[content["session_token"]]
                ):
                    del self.ws_server_instance.sessions[content["session_token"]]
                    setattr(websocket_protocol, "is_logged_in", False)
                else:
                    self.log(
                        "The user {} wanted to quit with a fake session token.".format(
                            content["username"]
                        ),
                        simple_ws_server_application_log_level.LEVEL_WARNING,
                    )
            except KeyError:
                try:
                    self.log(
                        "The user {} wanted to quit with a fake session token.".format(
                            content["username"]
                        ),
                        simple_ws_server_application_log_level.LEVEL_WARNING,
                    )
                except:
                    pass
            except AttributeError:
                pass
            except Exception as e:
                raise e

    def get_md5(self, data):
        import hashlib

        hash = hashlib.md5("add-some-salt".encode("utf-8"))
        hash.update(data.encode("utf-8"))
        return hash.hexdigest()

    async def on_register(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict,
    ):
        """
        Official callback method for registration
        """
        response = dict()
        username = content["username"]
        password = content["password"]
        password_hash = self.get_md5(password)
        self.log(
            "The user {} try to register with hash: {}.".format(username, password_hash)
        )
        self.ws_server_instance.server_instance.get_module_instance(
            "db_connector"
        ).database_cursor.execute(
            f'SELECT password FROM users WHERE username = "{username}";'
        )
        tmp = self.ws_server_instance.server_instance.get_module_instance(
            "db_connector"
        ).database_cursor.fetchone()
        if tmp == None:
            try:
                self.ws_server_instance.server_instance.get_module_instance(
                    "db_connector"
                ).database_cursor.execute(
                    f'INSERT INTO users (username, password) VALUES ("{username}", "{password_hash}");'
                )
                self.ws_server_instance.server_instance.get_module_instance(
                    "db_connector"
                ).database.commit()
                self.log("The user {} registered successfully.".format(username))
            except:
                self.ws_server_instance.server_instance.get_module_instance(
                    "db_connector"
                ).database.rollback()

            response = {
                "type": "quit",
                "content": {
                    "reason": "registration_success",
                    "request_key": content["request_key"],
                },
            }
            await websocket_protocol.send(json.dumps(response))
            response.clear()
        else:
            self.log(
                "The user {} failed to register.".format(username),
                simple_ws_server_application_log_level.LEVEL_WARNING,
            )
            response = {
                "type": "quit",
                "content": {
                    "reason": "registration_failure",
                    "request_key": content["request_key"],
                },
            }
            await websocket_protocol.send(json.dumps(response))
            response.clear()

    async def on_online_user(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict,
    ):
        response = dict()
        response["type"] = "online_user"
        response["content"] = {
            "online_users": [],
            "request_key": content["request_key"],
        }
        for session_username in self.ws_server_instance.sessions.values():
            response["content"]["online_users"].append(session_username)

        await websocket_protocol.send(json.dumps(response))
        response.clear()
