"""
Note that this file is deprecated. Just use it for the purpose of documentation. :)
"""

import os, sys, json, logging, platform

try:
    import asyncio, nest_asyncio, websockets, textual.app, textual.events, textual.screen, textual.widgets
except ImportError:
    print("Installing dependencies...")
    os.system("pip install asyncio nest-asyncio websockets textual")
    try:
        import asyncio, nest_asyncio, websockets, textual.app, textual.events, textual.screen, textual.widgets
    except ImportError:
        print("Dependencies installation failed.")
        sys.exit(-1)

SERVER_HOST: str = "ws://localhost:9982"  # Test server address

session_token: str = ""
login_username: str = ""
login_password: str = ""
register_username: str = ""
register_password: str = ""
is_processing: bool = True
is_logged: bool = False
chat_env: bool = False
server_down: bool = False
ws: websockets.WebSocketClientProtocol
background_tasks: list[asyncio.Task] = []

logging.getLogger("asyncio").setLevel(logging.DEBUG)
nest_asyncio.apply()


async def message_processing(websocket_protocol: websockets.WebSocketClientProtocol):
    global is_logged, server_down, is_processing, session_token

    client_log: textual.widgets.Log = application.query_one(
        "#client_log", textual.widgets.Log
    )
    while True:
        await asyncio.sleep(0)
        async for original_message in websocket_protocol:
            try:
                is_processing = True
                message = json.loads(original_message)
                if len(original_message) < 200:
                    client_log.write_line(original_message)
                else:
                    client_log.write_line("TLDR: {}".format(type(message)))

                if message["type"] == "problem_set":
                    client_log.write_line("+ Problem Set")
                    for index, problem_number in enumerate(message["problem_set"]):
                        if index == 0:
                            client_log.write_line(problem_number + " ")
                        else:
                            client_log.write(problem_number + " ")

                    client_log.write_line("- Problem Set")
                    is_processing = False

                elif message["type"] == "problem_statement":
                    client_log.write_line("+ Problem Statement")

                    client_log.write_line(
                        "Name: {} - {}".format(
                            message["problem_number"], message["problem_name"]
                        )
                    )
                    client_log.write_line(
                        "Difficulty: {}".format(message["difficulty"])
                    )
                    for line in message["problem_statement"]:

                        client_log.write_line(line)

                    client_log.write_line("- Problem Statement")
                    is_processing = False

                elif message["type"] == "submission_result":
                    client_log.write_line("+ Submission Result")
                    client_log.write_line("Result: {}".format(message["result"]))
                    if format(message["result"]) == "WA":
                        for reason in message["reasons"]:
                            client_log.write_line("Reason: {}".format(reason))

                    client_log.write_line("- Submission Result")
                    is_processing = False

                elif message["type"] == "session_token":
                    session_token = message["content"]
                    client_log.write_line("Session Token: {}".format(session_token))
                    is_logged = True
                    is_processing = False

                elif message["type"] == "online_user":
                    client_log.write_line("Online Users: {}".format(message["content"]))
                    is_processing = False

                elif message["type"] == "chat_message":
                    client_log.write_line("+ Chat Message")
                    client_log.write_line("From: {}".format(message["from"]))
                    chat_messages: list[str] = message["content"]
                    for chat_message in chat_messages:
                        client_log.write_line("chat> {}".format(chat_message))

                    client_log.write_line("- Chat Message")
                    is_processing = False

                elif message["type"] == "quit":
                    if message["content"] == "authentication_failure":
                        client_log.write_line("Authentication Failure")
                    if message["content"] == "registration_failure":
                        client_log.write_line("Registration Failure")
                    if message["content"] == "registration_success":
                        client_log.write_line("Registration Success")

                    is_processing = False
                    await websocket_protocol.close()
                    break
                elif message["type"] == "music_stream_head":
                    pass

            except Exception as e:
                logging.exception(e)
                is_processing = False
                await websocket_protocol.close()
                raise e

        await asyncio.sleep(0)


async def login_session(ws):
    global is_processing
    try:
        is_processing = True
        message = {
            "type": "login",
            "content": {"username": login_username, "password": login_password},
        }
        await ws.send(json.dumps(message))
        message.clear()
        application.push_screen(UserScreen())
    except Exception as e:
        logging.exception(e)


async def register_session(ws):
    global is_processing
    try:
        is_processing = True
        message = {
            "type": "register",
            "content": {"username": register_username, "password": register_password},
        }
        await ws.send(json.dumps(message))
        message.clear()
    except Exception as e:
        logging.exception(e)


async def websocket_session_on_close(ws, close_status_code, close_msg):
    global is_processing
    try:
        is_processing = True
        if session_token == "":
            message = {"type": "close_connection", "content": {}}
        else:
            message = {
                "type": "quit",
                "content": {"username": login_username, "session_token": session_token},
            }

        await ws.send(json.dumps(message))
        message.clear()
        is_processing = False
        sys.exit(0)
    except websockets.ConnectionClosed:
        sys.exit(0)
    except Exception as e:
        logging.exception(e)


async def websocket_session(on_open):
    global t, ws, server_down, background_tasks
    try:
        async with websockets.connect(uri=SERVER_HOST) as ws:
            try:
                await on_open(ws)
                task_message_processing = asyncio.create_task(message_processing(ws))
                background_tasks.append(task_message_processing)
                task_message_processing.add_done_callback(background_tasks.remove)

                await task_message_processing
            except websockets.exceptions.ConnectionClosed:
                server_down = True
                print("Quitting...")
                application.exit()
                sys.exit(0)
            except Exception as e:
                logging.exception(e)
                raise e
    except KeyboardInterrupt:
        print("Quitting...")
        application.exit()
        sys.exit(0)
        raise KeyboardInterrupt
    except Exception as e:
        logging.exception(e)
        raise e


async def input_processing(websocket_protocol: websockets.WebSocketClientProtocol):
    global is_processing, login_username, is_logged

    try:
        while is_processing:
            await asyncio.sleep(0)

        command = application.query_one("#command_input", textual.widgets.Input).value
        application.query_one("#command_input", textual.widgets.Input).clear()

        client_log: textual.widgets.Log = application.query_one(
            "#client_log", textual.widgets.Log
        )
        client_log.write_line(command)
        command = command.strip().split()
        if command == []:  # Empty command
            return

        if command[0] == "%help":  # Check help
            client_log.write_line("使用 %help 来查看本帮助")
            client_log.write_line("使用 %problem_set 来获取题目列表")
            client_log.write_line(
                "使用 %problem_statement[题目编号] 来获取指定题目信息"
            )
            client_log.write_line(
                "使用 %submit [题目编号] [源文件名] [代码语言] 来递交指定语言的代码测评指定题目"
            )
            client_log.write_line(
                "使用 %chat [短讯聊天对象] 来短讯聊天，此命令会使你进入短讯聊天环境，键入 %send 来确认发送，键入 %cancel 来取消发送"
            )
            client_log.write_line("使用 %online_user 来查询在线用户")
            client_log.write_line("使用 %clear 来清除输出")
            client_log.write_line("使用 %debug [on / off] 来开启或关闭调试模式")
            client_log.write_line("使用 %quit 或 %exit 退出")

        elif command[0] == "%problem_set":  # Get problem list
            is_processing = True
            message = {"type": "problem_set", "content": {}}
            await websocket_protocol.send(json.dumps(message))
            message.clear()

        elif command[0] == "%problem_statement":  # Get statement of a specific problem
            if len(command) < 2:
                client_log.write_line("Please specify which problem you want to show.")
                return

            is_processing = True
            message = {
                "type": "problem_statement",
                "content": {"problem_number": command[1]},
            }
            await websocket_protocol.send(json.dumps(message))
            message.clear()

        elif command[0] == "%submit":  # Submit source code for judgment
            if len(command) < 4:
                client_log.write_line("Bad format.")
                return

            problem_number = command[1]
            file_name = command[2]
            language = command[3]
            message = {
                "type": "submission",
                "content": {
                    "username": login_username,
                    "session_token": session_token,
                    "problem_number": problem_number,
                    "language": language,
                    "code": [],
                },
            }
            try:
                with open(file_name, "r") as file:
                    lines = file.readlines()
                    for line in lines:
                        message["content"]["code"].append(line)
            except FileNotFoundError:
                client_log.write_line("File {} not found.".format(file_name))
                return

            is_processing = True
            await websocket_protocol.send(json.dumps(message))
            message.clear()

        elif command[0] == "%online_user":  # Check online users
            is_processing = True
            message = {"type": "online_user", "content": {}}
            await websocket_protocol.send(json.dumps(message))
            message.clear()

        elif command[0] == "%chat":  # Send short chat message
            raise NotImplementedError(
                "Chat command for TUI Client is not implemented yet."
            )
            # if len(command) < 2:
            #     print('Please specify whom you want to send this message to.')
            #     return

            # is_processing = True;
            # try:
            #     chat_messages: list[str] = []
            #     while True:
            #         if chat_message == '%send' or chat_message == '%confirm':
            #             message = {
            #                 'type': 'chat_short',
            #                 'content': {
            #                     'from': login_username, 'to': command[1], 'messages': chat_messages, 'session_token': session_token
            #                 }
            #             }
            #             await websocket_protocol.send(json.dumps(message)); message.clear()
            #             break
            #         if chat_message == '%cancel':
            #             break

            #         chat_messages.append(chat_message)
            #         await asyncio.sleep(0)
            # except EOFError:
            #     is_processing = False
            #     is_processing = True
            #     if session_token == '':
            #         message = {'type': 'close_connection', 'content': {}}
            #     else:
            #         message = {'type': 'quit', 'content': {'username': login_username,'session_token': session_token}}

            #     await websocket_protocol.send(json.dumps(message)); message.clear() # Send quit message
            #     await websocket_protocol.close()
            #     is_processing = False
            #     sys.exit(0)
            # except Exception as e:
            #     logging.exception(e)

            # is_processing = False

        elif command[0] == "%clear":  # Clear screen
            if platform.system() == "Windows":
                os.system("cls")
            if platform.system() == "Linux":
                os.system("clear")

        elif command[0] == "%debug":  # Toggle debug mode
            if len(command) < 2:
                print(
                    "Please specify whether you want to turn on the debug output or not."
                )
                return

            if command[1] == "on":
                asyncio.get_event_loop_policy().get_event_loop().set_debug(True)
            if command[1] == "off":
                asyncio.get_event_loop_policy().get_event_loop().set_debug(False)

        elif command[0] == "%quit" or command[0] == "%exit":  # Quit the client program
            is_processing = True
            if session_token == "":
                message = {"type": "close_connection", "content": {}}
            else:
                message = {
                    "type": "quit",
                    "content": {
                        "username": login_username,
                        "session_token": session_token,
                    },
                }

            await websocket_protocol.send(json.dumps(message))
            message.clear()  # Send quit message
            await websocket_protocol.close()
            is_processing = False
            is_logged = False

    except KeyboardInterrupt as e:
        print("Quitting...")
        is_processing = True
        if session_token == "":
            message = {"type": "close_connection", "content": {}}
        else:
            message = {
                "type": "quit",
                "content": {"username": login_username, "session_token": session_token},
            }

        await websocket_protocol.send(json.dumps(message))
        message.clear()  # Send quit message
        await websocket_protocol.close()
        is_processing = False
        raise e
    except Exception as e:
        logging.exception(e)


class LoginScreen(textual.screen.Screen):

    def compose(self) -> textual.app.ComposeResult:
        yield textual.containers.Grid(
            textual.widgets.Label("Login Session"),
            textual.widgets.Input(
                placeholder="Login Username", id="login_username_input"
            ),
            textual.widgets.Input(
                placeholder="Login Password", password=True, id="login_password_input"
            ),
            textual.widgets.Button("Cancel", variant="error", id="login_cancel_button"),
            textual.widgets.Button("Confirm", id="login_confirm_button"),
            classes="dialog",
        )

    async def on_button_pressed(self, event: textual.widgets.Button.Pressed) -> None:
        global login_username, login_password
        if event.button.id == "login_confirm_button":
            login_username = self.query_one(
                "#login_username_input", textual.widgets.Input
            ).value
            login_password = self.query_one(
                "#login_password_input", textual.widgets.Input
            ).value
            asyncio.create_task(websocket_session(login_session))
            self.app.pop_screen()
        if event.button.id == "login_cancel_button":
            self.app.pop_screen()


class RegisterScreen(textual.screen.Screen):

    def compose(self) -> textual.app.ComposeResult:
        yield textual.containers.Grid(
            textual.widgets.Label("Register Session"),
            textual.widgets.Input(
                placeholder="Register Username", id="register_username_input"
            ),
            textual.widgets.Input(
                placeholder="Register Password",
                password=True,
                id="register_password_input",
            ),
            textual.widgets.Button(
                "Cancel", variant="error", id="register_cancel_button"
            ),
            textual.widgets.Button("Confirm", id="register_confirm_button"),
            classes="dialog",
        )

    async def on_button_pressed(self, event: textual.widgets.Button.Pressed) -> None:
        global register_username, register_password
        if event.button.id == "register_confirm_button":
            register_username = self.query_one(
                "#register_username_input", textual.widgets.Input
            ).value
            register_password = self.query_one(
                "#register_password_input", textual.widgets.Input
            ).value
            asyncio.create_task(websocket_session(register_session))
            self.app.pop_screen()
        if event.button.id == "register_cancel_button":
            self.app.pop_screen()


class UserScreen(textual.screen.Screen):

    def compose(self) -> textual.app.ComposeResult:
        yield textual.containers.Grid(
            textual.widgets.Label("Genshin OJ Client (Main)"),
            textual.widgets.Input(placeholder="Command Input", id="command_input"),
            textual.widgets.Button("Exit", variant="error", id="command_exit_button"),
            textual.widgets.Button("Confirm", id="command_confirm_button"),
            classes="dialog",
        )
        yield textual.widgets.Log("Log Here...", id="client_log")

    async def on_button_pressed(self, event: textual.widgets.Button.Pressed) -> None:
        global login_username, is_logged, is_processing
        if event.button.id == "command_confirm_button":
            await input_processing(ws)
            if not is_logged:
                self.log("Quitting from login status...")
                self.app.exit()
                sys.exit(0)

        if event.button.id == "command_exit_button":
            is_logged = False
            is_processing = True

            try:
                if session_token == "":
                    message = {"type": "close_connection", "content": {}}
                else:
                    message = {
                        "type": "quit",
                        "content": {
                            "username": login_username,
                            "session_token": session_token,
                        },
                    }

                await ws.send(json.dumps(message))
                message.clear()  # Send quit message
                await ws.close()
                self.log("WS Connection Closed")
            except:
                raise

            is_processing = False
            self.app.exit()


class GenshinOJClient(textual.app.App):
    CSS_PATH = "client_tui.tcss"
    BINDINGS = [("d", "toggle_dark", "Toggle dark mode")]

    def compose(self) -> textual.app.ComposeResult:
        self.title = "Genshin OJ Client (TUI)"
        self.sub_title = "Startup Session"
        self.id = "main_tui_app"
        self.buttonLogin = textual.widgets.Button("Login", id="login")
        self.buttonRegister = textual.widgets.Button("Register", id="register")
        self.buttonQuit = textual.widgets.Button("Quit", id="quit", variant="error")
        yield textual.widgets.Header()
        with textual.containers.Container(
            id="main-container", classes="aligned"
        ) as self.main_container:
            yield self.buttonLogin
            yield self.buttonRegister
            yield self.buttonQuit

        yield textual.widgets.Footer()

    def action_toggle_dark(self) -> None:
        self.dark = not self.dark

    async def on_unmount(self, event: textual.events.Unmount) -> None:
        global ws
        try:

            if session_token == "":
                message = {"type": "close_connection", "content": {}}
            else:
                message = {
                    "type": "quit",
                    "content": {
                        "username": login_username,
                        "session_token": session_token,
                    },
                }

            try:
                await ws.send(json.dumps(message))
                message.clear()
                await ws.close()
            except NameError:
                pass
            except websockets.exceptions.ConnectionClosedOK:
                pass
            except:
                raise

            for background_task in background_tasks:
                background_task.cancel()
                await asyncio.wait([background_task])
                try:
                    await background_task
                except asyncio.CancelledError:
                    pass

            if len(background_tasks) > 0:
                self.log("Waiting for background tasks.")
        except:
            raise

        self.log("Client exited successfully.")

    @textual.on(textual.widgets.Button.Pressed, "#login")
    async def on_buttonLogin_pressed(
        self, event: textual.widgets.Button.Pressed
    ) -> None:
        global is_logged
        self.push_screen(LoginScreen())

    @textual.on(textual.widgets.Button.Pressed, "#register")
    async def on_buttonRegister_pressed(
        self, event: textual.widgets.Button.Pressed
    ) -> None:
        self.push_screen(RegisterScreen())

    @textual.on(textual.widgets.Button.Pressed, "#quit")
    def on_buttonQuit_pressed(self, event: textual.widgets.Button.Pressed) -> None:
        self.exit()


async def main():
    t = asyncio.create_task(application.run_async())
    while True:
        await asyncio.sleep(0)


if __name__ == "__main__":
    try:
        application = GenshinOJClient()
        asyncio.run(main())
        asyncio.get_event_loop().run_forever()
    except KeyboardInterrupt as e:
        sys.exit(0)
        raise e
