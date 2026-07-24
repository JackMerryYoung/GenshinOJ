#!/usr/bin/env python3
"""One-click database reset for RsOJ.

Drops the entire `RsOJ` database and recreates it empty. The server
recreates all tables (`users`, `judge`, `judge_user_submission`,
`judge_submission_result`, ...) on next startup via `CREATE TABLE IF NOT EXISTS`,
so an empty database is all that's needed.

Usage:
    python clear_database.py        # asks for confirmation
    python clear_database.py -y     # skip confirmation (true one-click)

Connection defaults match db_connector/db_connector.py and can be overridden
with the DB_USER / DB_PASSWORD / DB_PORT / DB_HOST environment variables.
"""

import os
import sys

try:
    import pymysql
except ImportError:
    print("Installing dependencies...")
    os.system("pip install pymysql cryptography")
    import pymysql

DATABASE_NAME = "RsOJ"
DATABASE_HOST = os.environ.get("DB_HOST", "localhost")
DATABASE_USER = os.environ.get("DB_USER", "root")
DATABASE_PASSWORD = os.environ.get("DB_PASSWORD", "123456")
DATABASE_PORT = int(os.environ.get("DB_PORT", "3306"))


def main() -> int:
    skip_confirm = any(arg in ("-y", "--yes") for arg in sys.argv[1:])

    if not skip_confirm:
        print(
            f"\033[1;33mThis will DROP the entire `{DATABASE_NAME}` database "
            f"on {DATABASE_HOST}:{DATABASE_PORT} and recreate it empty.\033[0m"
        )
        answer = input(f"Type the database name (`{DATABASE_NAME}`) to confirm: ")
        if answer.strip() != DATABASE_NAME:
            print("Aborted. Nothing was changed.")
            return 1

    try:
        connection = pymysql.connect(
            host=DATABASE_HOST,
            user=DATABASE_USER,
            passwd=DATABASE_PASSWORD,
            port=DATABASE_PORT,
        )
    except Exception as error:
        print(f"\033[1;31mFailed to connect to MySQL: {error}\033[0m")
        return 1

    try:
        with connection.cursor() as cursor:
            cursor.execute(f"DROP DATABASE IF EXISTS {DATABASE_NAME};")
            cursor.execute(f"CREATE DATABASE {DATABASE_NAME};")
        connection.commit()
    finally:
        connection.close()

    print(
        f"\033[1;32mDatabase `{DATABASE_NAME}` cleared. "
        f"Tables will be rebuilt on the next server startup.\033[0m"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
