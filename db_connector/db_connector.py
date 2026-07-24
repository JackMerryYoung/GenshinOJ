import os, sys, enum

try:
    import pymysql
    import pymysql.connections
except:
    print("Installing dependencies...")
    os.system("pip install pymysql cryptography")
    try:
        import pymysql
        import pymysql.connections
    except:
        print("Dependencies installation failed.")
        os.exit(-1)

import server


class db_connector_log_level(enum.Enum):
    LEVEL_INFO = 0
    LEVEL_DEBUG = 1
    LEVEL_WARNING = 2
    LEVEL_ERROR = 3


class db_connector:

    def __init__(self, server_instance: server.server) -> None:
        # Necessary Initialization
        self.server_instance = server_instance
        self.server_instance.working_loads["db_connector"]["instance"] = self
        self.database: pymysql.Connection
        self.database_cursor: pymysql.Cursor

        self.DatabaseConnection()

    def on_unload(self) -> None:
        self.log("Database connector unloaded.")

    def log(
        self,
        log: str,
        log_level: db_connector_log_level = db_connector_log_level.LEVEL_INFO,
    ):
        call_frame = sys._getframe(1)
        if log_level is db_connector_log_level.LEVEL_INFO:
            print(
                "\033[1;2m[DB_CONNECTOR] [INFO] {} (file `{}`, function `{}` on line {})\033[0m".format(
                    log,
                    call_frame.f_code.co_filename,
                    call_frame.f_code.co_name,
                    call_frame.f_lineno,
                )
            )
        if log_level is db_connector_log_level.LEVEL_DEBUG:
            print(
                "\033[1;34m[DB_CONNECTOR] [JUDGE_WS_SERVER_APP] [DEBUG] {} (file `{}`, function `{}` on line {})\033[0m".format(
                    log,
                    call_frame.f_code.co_filename,
                    call_frame.f_code.co_name,
                    call_frame.f_lineno,
                )
            )
        if log_level is db_connector_log_level.LEVEL_WARNING:
            print(
                "\033[1;33m[DB_CONNECTOR] [JUDGE_WS_SERVER_APP] [WARNING] {} (file `{}`, function `{}` on line {})\033[0m".format(
                    log,
                    call_frame.f_code.co_filename,
                    call_frame.f_code.co_name,
                    call_frame.f_lineno,
                )
            )
        if log_level is db_connector_log_level.LEVEL_ERROR:
            print(
                "\033[1;31m[DB_CONNECTOR] [JUDGE_WS_SERVER_APP] [ERROR] {} (file `{}`, function `{}` on line {})\033[0m".format(
                    log,
                    call_frame.f_code.co_filename,
                    call_frame.f_code.co_name,
                    call_frame.f_lineno,
                )
            )

    def DatabaseConnection(
        self, DATABASE_USER="root", DATABASE_PASSWORD="123456", DATABASE_PORT=3306
    ):  # Database Connection
        try:  # Connect to database
            self.database: pymysql.Connection = pymysql.connect(
                host="localhost",
                user=DATABASE_USER,
                passwd=DATABASE_PASSWORD,
                port=DATABASE_PORT,
                database="RsOJ",
            )
            self.log("Connection established.")
        except Exception:  # If the database is not established, create it
            self.database: pymysql.Connection = pymysql.connect(
                host="localhost",
                user=DATABASE_USER,
                passwd=DATABASE_PASSWORD,
                port=DATABASE_PORT,
            )
            self.database_cursor = self.database.cursor()
            self.database_cursor.execute("CREATE DATABASE IF NOT EXISTS RsOJ;")
            self.database = pymysql.connect(
                host="localhost",
                user=DATABASE_USER,
                passwd=DATABASE_PASSWORD,
                port=DATABASE_PORT,
                database="RsOJ",
            )
            self.log("Initialization finished!")

        # Get MySQL's version
        self.database_cursor = self.database.cursor()
        self.database_cursor.execute("SELECT VERSION()")
        self.log(f"MySQL Version: {self.database_cursor.fetchone()[0]}")
