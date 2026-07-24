#!/usr/bin/env python3
"""
Set a user as administrator in RsOJ
"""

import mysql.connector
import sys

DB_CONFIG = {
    'host': 'localhost',
    'user': 'root',
    'password': '123456',
    'database': 'RsOJ'
}

def set_admin(username):
    """Set a user as administrator"""
    try:
        conn = mysql.connector.connect(**DB_CONFIG)
        cursor = conn.cursor()

        # Check if user exists
        cursor.execute("SELECT id, username, is_admin FROM users WHERE username = %s", (username,))
        user = cursor.fetchone()

        if not user:
            print(f"❌ Error: User '{username}' not found")
            return False

        user_id, username, is_admin = user

        if is_admin == 1:
            print(f"ℹ️  User '{username}' is already an administrator")
            return True

        # Set user as admin
        cursor.execute("UPDATE users SET is_admin = 1 WHERE username = %s", (username,))
        conn.commit()

        print(f"✅ User '{username}' (ID: {user_id}) is now an administrator")
        return True

    except mysql.connector.Error as e:
        print(f"❌ Database error: {e}")
        return False
    finally:
        if conn:
            cursor.close()
            conn.close()

def remove_admin(username):
    """Remove admin privileges from a user"""
    try:
        conn = mysql.connector.connect(**DB_CONFIG)
        cursor = conn.cursor()

        # Check if user exists
        cursor.execute("SELECT id, username, is_admin FROM users WHERE username = %s", (username,))
        user = cursor.fetchone()

        if not user:
            print(f"❌ Error: User '{username}' not found")
            return False

        user_id, username, is_admin = user

        if is_admin == 0:
            print(f"ℹ️  User '{username}' is not an administrator")
            return True

        # Remove admin privileges
        cursor.execute("UPDATE users SET is_admin = 0 WHERE username = %s", (username,))
        conn.commit()

        print(f"✅ Admin privileges removed from user '{username}' (ID: {user_id})")
        return True

    except mysql.connector.Error as e:
        print(f"❌ Database error: {e}")
        return False
    finally:
        if conn:
            cursor.close()
            conn.close()

def list_admins():
    """List all administrators"""
    try:
        conn = mysql.connector.connect(**DB_CONFIG)
        cursor = conn.cursor()

        cursor.execute("SELECT id, username, created_at FROM users WHERE is_admin = 1 ORDER BY id")
        admins = cursor.fetchall()

        if not admins:
            print("ℹ️  No administrators found")
            return

        print("\n📋 Administrators:")
        print("-" * 60)
        print(f"{'ID':<8} {'Username':<30} {'Created At':<20}")
        print("-" * 60)

        for user_id, username, created_at in admins:
            from datetime import datetime
            created = datetime.fromtimestamp(created_at / 1000).strftime('%Y-%m-%d %H:%M:%S')
            print(f"{user_id:<8} {username:<30} {created:<20}")

        print("-" * 60)
        print(f"Total: {len(admins)} administrator(s)\n")

    except mysql.connector.Error as e:
        print(f"❌ Database error: {e}")
    finally:
        if conn:
            cursor.close()
            conn.close()

def main():
    if len(sys.argv) < 2:
        print("Usage:")
        print("  python manage_admin.py add <username>      - Set user as administrator")
        print("  python manage_admin.py remove <username>   - Remove admin privileges")
        print("  python manage_admin.py list                - List all administrators")
        sys.exit(1)

    command = sys.argv[1].lower()

    if command == "list":
        list_admins()
    elif command in ["add", "set", "grant"]:
        if len(sys.argv) < 3:
            print("❌ Error: Username is required")
            print("Usage: python manage_admin.py add <username>")
            sys.exit(1)
        username = sys.argv[2]
        set_admin(username)
    elif command in ["remove", "revoke", "delete"]:
        if len(sys.argv) < 3:
            print("❌ Error: Username is required")
            print("Usage: python manage_admin.py remove <username>")
            sys.exit(1)
        username = sys.argv[2]
        remove_admin(username)
    else:
        print(f"❌ Error: Unknown command '{command}'")
        print("Valid commands: add, remove, list")
        sys.exit(1)

if __name__ == "__main__":
    main()
