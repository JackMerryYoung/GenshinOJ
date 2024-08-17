import os
import platform
import sys, json, enum, hashlib

import websockets

from ... import ws_server


class judge_ws_server_log_level(enum.Enum):
    LEVEL_INFO = 0
    LEVEL_DEBUG = 1
    LEVEL_WARNING = 2
    LEVEL_ERROR = 3


class judge_ws_server_application(ws_server.ws_server_application_protocol):

    def __init__(self, ws_server_instance: ws_server) -> None:
        super().__init__(ws_server_instance)
        if platform.system() == "Windows":
            JUDGE_WS_SERVER_APPLICATION_CONFIG_JSON_PATH = (
                os.getcwd()
                + "\\ws_server\\ws_server_applications\\judge_ws_server_application\\config.json"
            )
        if platform.system() == "Linux":
            JUDGE_WS_SERVER_APPLICATION_CONFIG_JSON_PATH = (
                os.getcwd()
                + "/ws_server/ws_server_applications/judge_ws_server_application/config.json"
            )
        with open(
            JUDGE_WS_SERVER_APPLICATION_CONFIG_JSON_PATH,
            "r",
        ) as judge_ws_server_application_config_json_file:
            self.judge_ws_server_application_config = json.load(
                judge_ws_server_application_config_json_file
            )

            self.log(self.judge_ws_server_application_config)

    def log(
        self,
        log: str,
        log_level: judge_ws_server_log_level = judge_ws_server_log_level.LEVEL_INFO,
    ):
        call_frame = sys._getframe(1)
        if log_level is judge_ws_server_log_level.LEVEL_INFO:
            print(
                "\033[1;2m[WS_SERVER] [JUDGE_WS_SERVER_APP] [INFO] {} (file `{}`, function `{}` on line {})\033[0m".format(
                    log,
                    call_frame.f_code.co_filename,
                    call_frame.f_code.co_name,
                    call_frame.f_lineno,
                )
            )
        if log_level is judge_ws_server_log_level.LEVEL_DEBUG:
            print(
                "\033[1;34m[WS_SERVER] [JUDGE_WS_SERVER_APP] [DEBUG] {} (file `{}`, function `{}` on line {})\033[0m".format(
                    log,
                    call_frame.f_code.co_filename,
                    call_frame.f_code.co_name,
                    call_frame.f_lineno,
                )
            )
        if log_level is judge_ws_server_log_level.LEVEL_WARNING:
            print(
                "\033[1;33m[WS_SERVER] [JUDGE_WS_SERVER_APP] [WARNING] {} (file `{}`, function `{}` on line {})\033[0m".format(
                    log,
                    call_frame.f_code.co_filename,
                    call_frame.f_code.co_name,
                    call_frame.f_lineno,
                )
            )
        if log_level is judge_ws_server_log_level.LEVEL_ERROR:
            print(
                "\033[1;31m[WS_SERVER] [JUDGE_WS_SERVER_APP] [ERROR] {} (file `{}`, function `{}` on line {})\033[0m".format(
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

    async def on_close_connection(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict = None,
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

    def get_md5(self, data):
        hash = hashlib.md5("add-some-salt".encode("utf-8"))
        hash.update(data.encode("utf-8"))
        return hash.hexdigest()

    async def on_submission(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict,
    ):
        try:
            self.ws_server_instance.server_instance.get_module_instance(
                "judge"
            ).now_submission_id = (
                self.ws_server_instance.server_instance.get_module_instance(
                    "judge"
                ).now_submission_id
                + 1
            )

            now_submission_id = (
                self.ws_server_instance.server_instance.get_module_instance(
                    "judge"
                ).now_submission_id
            )

            self.log("Processing Submission {}".format(now_submission_id))
            self.ws_server_instance.server_instance.get_module_instance(
                "judge"
            ).judgment_queue.append(
                {
                    "submission_id": now_submission_id,
                    "problem_number": int(content["problem_number"]),
                    "language": content["language"],
                    "username": self.ws_server_instance.server_instance.get_module_instance(
                        "ws_server"
                    ).sessions[
                        content["session_token"]
                    ],
                    "code": content["code"],
                    "websocket_protocol": websocket_protocol,
                }
            )

            try:
                response = {
                    "type": "submission_id",
                    "content": {
                        "submission_id": now_submission_id,
                        "request_key": content["request_key"],
                    },
                }
                await websocket_protocol.send(json.dumps(response))
            except websockets.exceptions.ConnectionClosed:
                pass

            response.clear()
        except Exception as e:
            self.log(repr(e), judge_ws_server_log_level.LEVEL_WARNING)

    async def on_problem_statement(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict,
    ):
        try:
            response = dict()
            response["type"] = "problem_statement"
            response["content"] = dict()
            try:
                with open(
                    self.ws_server_instance.server_instance.get_module_instance(
                        "judge"
                    ).get_problem_statement_json_path(int(content["problem_number"])),
                    "r",
                ) as problem_statement_json_file:
                    response["content"].update(json.load(problem_statement_json_file))

                response["content"].update({"request_key": content["request_key"]})
                await websocket_protocol.send(json.dumps(response))
                response.clear()
            except FileNotFoundError as e:
                self.log(
                    "problem_statement.json({}) is not found!".format(
                        self.ws_server_instance.server_instance.get_module_instance(
                            "judge"
                        ).get_problem_statement_json_path(
                            int(content["problem_number"])
                        )
                    ),
                    judge_ws_server_log_level.LEVEL_ERROR,
                )
                response["content"].update(
                    {
                        "problem_number": -1,
                        "difficulty": -1,
                        "problem_name": "Problem Not Found",
                        "problem_statement": [
                            "You tried to request a problem not existed."
                        ],
                        "request_key": content["request_key"],
                    }
                )
                await websocket_protocol.send(json.dumps(response))
                response.clear()
        except Exception as e:
            self.log(repr(e), judge_ws_server_log_level.LEVEL_WARNING)

    async def on_problem_set(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict,
    ):

        try:
            response = dict()
            response["type"] = "problem_set"
            response["content"] = dict()
            response["content"]["request_key"] = content["request_key"]
            try:
                with open(
                    self.ws_server_instance.server_instance.get_module_instance(
                        "judge"
                    ).get_problem_set_json_path(),
                    "r",
                ) as problem_set_json_file:
                    response["content"]["problem_set"] = json.load(
                        problem_set_json_file
                    )["problem_set"]
            except OSError:
                self.log(
                    "problem_set.json({}) is not found!".format(
                        self.ws_server_instance.server_instance.get_module_instance(
                            "judge"
                        ).get_problem_set_json_path()
                    ),
                    judge_ws_server_log_level.LEVEL_ERROR,
                )

            await websocket_protocol.send(json.dumps(response))
            response.clear()
        except Exception as e:
            self.log(repr(e), judge_ws_server_log_level.LEVEL_WARNING)

    async def on_submission_result(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict,
    ):
        try:
            response = dict()
            response["type"] = "submission_result"
            response["content"] = dict()
            if (
                content["submission_id"]
                > self.ws_server_instance.server_instance.get_module_instance(
                    "judge"
                ).now_submission_id
                or content["submission_id"] <= 0
            ):
                self.log(
                    "Submission {} is not found!".format(content["submission_id"]),
                    judge_ws_server_log_level.LEVEL_WARNING,
                )
                response["content"]["result"] = "SNF"
                response["content"]["submission_id"] = content["submission_id"]
                response["content"]["request_key"] = content["request_key"]
                await websocket_protocol.send(json.dumps(response))
                response.clear()
                return

            self.ws_server_instance.server_instance.get_module_instance(
                "db_connector"
            ).database_cursor.execute(
                "SELECT result FROM judge_submission_result WHERE submission_id = {};".format(
                    content["submission_id"]
                )
            )

            fetch_results = self.ws_server_instance.server_instance.get_module_instance(
                "db_connector"
            ).database_cursor.fetchone()

            response["content"] = json.loads(fetch_results[0])
            response["content"]["request_key"] = content["request_key"]
            await websocket_protocol.send(json.dumps(response))
            response.clear()
        except Exception as e:
            self.log(repr(e), judge_ws_server_log_level.WARNING)

    async def on_submissions_list(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict,
    ):
        now_submission_id = self.ws_server_instance.server_instance.get_module_instance(
            "judge"
        ).now_submission_id
        self.ws_server_instance.server_instance.get_module_instance(
            "db_connector"
        ).database_cursor.execute(
            "SELECT result FROM judge_submission_result WHERE submission_id >= {} AND submission_id < {} ORDER BY submission_id DESC;".format(
                now_submission_id
                - content["index"]
                * self.judge_ws_server_application_config["page_size"]
                + 1,
                now_submission_id
                - (content["index"] - 1)
                * self.judge_ws_server_application_config["page_size"]
                + 1,
            )
        )

        response = dict()
        response["type"] = "submissions_list"
        response["content"] = dict()
        response["content"]["request_key"] = content["request_key"]
        response["content"]["submissions_list"] = list()
        fetch_results = self.ws_server_instance.server_instance.get_module_instance(
            "db_connector"
        ).database_cursor.fetchall()
        if fetch_results != None:
            for fetch_result in fetch_results:
                response["content"]["submissions_list"].append(
                    json.loads(fetch_result[0])
                )

        await websocket_protocol.send(json.dumps(response))
        response.clear()

    async def on_total_submissions_list_index(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict,
    ):
        import math

        now_submission_id = self.ws_server_instance.server_instance.get_module_instance(
            "judge"
        ).now_submission_id
        response = dict()
        response["type"] = "total_submissions_list_index"
        response["content"] = dict()
        response["content"]["request_key"] = content["request_key"]
        response["content"]["total_submissions_list_index"] = int(
            math.ceil(
                now_submission_id / self.judge_ws_server_application_config["page_size"]
            )
        )
        await websocket_protocol.send(json.dumps(response))
        response.clear()

    async def on_user_profile(
        self,
        websocket_protocol: websockets.server.WebSocketServerProtocol,
        content: dict,
    ):
        response = dict()
        response["type"] = "user_profile"
        response["content"] = dict()
        response["content"]["request_key"] = content["request_key"]
        self.ws_server_instance.server_instance.get_module_instance(
            "db_connector"
        ).database_cursor.execute(
            "SELECT accepted, test_accepted, general FROM judge_user_submission WHERE username = '{}';".format(
                content["username"]
            )
        )
        fetch_result = self.ws_server_instance.server_instance.get_module_instance(
            "db_connector"
        ).database_cursor.fetchone()
        response["content"]["accepted"] = fetch_result[0]
        response["content"]["test_accepted"] = fetch_result[1]
        response["content"]["general"] = fetch_result[2]

        await websocket_protocol.send(json.dumps(response))
        response.clear()
