#!/usr/bin/env python3
"""
Migrate problem configurations from JSON files to database.

This script reads problem_statement.json and problem_testcase_config.json
from the problem/ directory and imports them into the RsOJ database.
"""

import os
import sys
import json
import pymysql

DATABASE_NAME = "RsOJ"
DATABASE_HOST = os.environ.get("DB_HOST", "localhost")
DATABASE_USER = os.environ.get("DB_USER", "root")
DATABASE_PASSWORD = os.environ.get("DB_PASSWORD", "123456")
DATABASE_PORT = int(os.environ.get("DB_PORT", "3306"))

PROBLEM_DIR = os.path.join(os.path.dirname(__file__), "problem")


def main():
    # Connect to database
    try:
        connection = pymysql.connect(
            host=DATABASE_HOST,
            user=DATABASE_USER,
            passwd=DATABASE_PASSWORD,
            port=DATABASE_PORT,
            database=DATABASE_NAME,
        )
    except Exception as error:
        print(f"\033[1;31mFailed to connect to MySQL: {error}\033[0m")
        return 1

    try:
        with connection.cursor() as cursor:
            # Find all problem directories
            problem_dirs = [
                d for d in os.listdir(PROBLEM_DIR)
                if os.path.isdir(os.path.join(PROBLEM_DIR, d)) and d.isdigit()
            ]

            imported_count = 0
            skipped_count = 0

            for problem_num in sorted(problem_dirs):
                problem_path = os.path.join(PROBLEM_DIR, problem_num)
                statement_file = os.path.join(problem_path, "problem_statement.json")
                testcase_file = os.path.join(problem_path, "problem_testcase_config.json")

                # Check if problem already exists in database
                cursor.execute(
                    f"SELECT COUNT(*) FROM problems WHERE problem_number = %s",
                    (int(problem_num),)
                )
                exists = cursor.fetchone()[0] > 0

                if exists:
                    print(f"⏭️  Problem {problem_num} already exists in database, skipping...")
                    skipped_count += 1
                    continue

                # Read problem statement
                if not os.path.exists(statement_file):
                    print(f"⚠️  Problem {problem_num}: statement file not found, skipping...")
                    continue

                with open(statement_file, 'r', encoding='utf-8') as f:
                    statement_data = json.load(f)

                problem_number = statement_data.get("problem_number")
                problem_name = statement_data.get("problem_name")
                difficulty = statement_data.get("difficulty", 1)
                problem_statement = json.dumps(statement_data.get("problem_statement", []))

                # Read testcase config (optional)
                testcase_config = None
                if os.path.exists(testcase_file):
                    with open(testcase_file, 'r', encoding='utf-8') as f:
                        testcase_config = f.read()

                # Insert into database
                cursor.execute(
                    """INSERT INTO problems
                       (problem_number, problem_name, difficulty, problem_statement, testcase_config)
                       VALUES (%s, %s, %s, %s, %s)""",
                    (problem_number, problem_name, difficulty, problem_statement, testcase_config)
                )

                print(f"✅ Imported problem {problem_num}: {problem_name}")
                imported_count += 1

        connection.commit()
        print(f"\n\033[1;32m✨ Migration complete!\033[0m")
        print(f"   Imported: {imported_count}")
        print(f"   Skipped: {skipped_count}")

    finally:
        connection.close()

    return 0


if __name__ == "__main__":
    try:
        import pymysql
    except ImportError:
        print("Installing dependencies...")
        os.system("pip install pymysql")
        import pymysql

    sys.exit(main())
