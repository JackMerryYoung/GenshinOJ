#!/bin/bash
# Manage administrators in RsOJ with role-based access control

DB_USER="root"
DB_PASS="123456"
DB_NAME="RsOJ"

set_admin() {
    local username="$1"
    local role="$2"

    if [ -z "$username" ]; then
        echo "❌ Error: Username is required"
        exit 1
    fi

    if [ -z "$role" ]; then
        echo "❌ Error: Role is required"
        echo "Available roles:"
        echo "  - problem_admin: Dashboard and problem management"
        echo "  - community_admin: Dashboard only (moderation APIs are not implemented yet)"
        echo "  - super_admin: All currently implemented control-panel APIs"
        exit 1
    fi

    # Validate role
    if [[ ! "$role" =~ ^(problem_admin|community_admin|super_admin)$ ]]; then
        echo "❌ Error: Invalid role '$role'"
        echo "Valid roles: problem_admin, community_admin, super_admin"
        exit 1
    fi

    # Check if user exists
    local exists=$(mysql -u"$DB_USER" -p"$DB_PASS" "$DB_NAME" -sN -e "SELECT COUNT(*) FROM users WHERE username='$username'" 2>/dev/null)
    if [ "$exists" -eq 0 ]; then
        echo "❌ Error: User '$username' not found"
        exit 1
    fi

    # Check current role
    local current_role=$(mysql -u"$DB_USER" -p"$DB_PASS" "$DB_NAME" -sN -e "SELECT admin_role FROM users WHERE username='$username'" 2>/dev/null)
    if [ "$current_role" = "$role" ]; then
        echo "ℹ️  User '$username' already has role '$role'"
        exit 0
    fi

    # Set role
    mysql -u"$DB_USER" -p"$DB_PASS" "$DB_NAME" -e "UPDATE users SET admin_role='$role' WHERE username='$username'" 2>/dev/null
    echo "✅ User '$username' is now a $role"
}

remove_admin() {
    local username="$1"
    if [ -z "$username" ]; then
        echo "❌ Error: Username is required"
        exit 1
    fi

    # Check if user exists
    local exists=$(mysql -u"$DB_USER" -p"$DB_PASS" "$DB_NAME" -sN -e "SELECT COUNT(*) FROM users WHERE username='$username'" 2>/dev/null)
    if [ "$exists" -eq 0 ]; then
        echo "❌ Error: User '$username' not found"
        exit 1
    fi

    # Remove admin role
    mysql -u"$DB_USER" -p"$DB_PASS" "$DB_NAME" -e "UPDATE users SET admin_role=NULL WHERE username='$username'" 2>/dev/null
    echo "✅ Admin privileges removed from user '$username'"
}

list_admins() {
    echo ""
    echo "📋 Administrators by Role:"
    echo "============================================================"

    # Super Admins
    echo ""
    echo "🔴 Super Administrators (Full Access):"
    echo "------------------------------------------------------------"
    mysql -u"$DB_USER" -p"$DB_PASS" "$DB_NAME" -e "SELECT id, username, FROM_UNIXTIME(created_at/1000) AS created_at FROM users WHERE admin_role='super_admin' ORDER BY id" 2>/dev/null

    # Problem Admins
    echo ""
    echo "🟢 Problem Administrators (Dashboard and Problems):"
    echo "------------------------------------------------------------"
    mysql -u"$DB_USER" -p"$DB_PASS" "$DB_NAME" -e "SELECT id, username, FROM_UNIXTIME(created_at/1000) AS created_at FROM users WHERE admin_role='problem_admin' ORDER BY id" 2>/dev/null

    # Community Admins
    echo ""
    echo "🔵 Community Administrators (Dashboard; moderation pending):"
    echo "------------------------------------------------------------"
    mysql -u"$DB_USER" -p"$DB_PASS" "$DB_NAME" -e "SELECT id, username, FROM_UNIXTIME(created_at/1000) AS created_at FROM users WHERE admin_role='community_admin' ORDER BY id" 2>/dev/null

    echo ""
}

show_help() {
    cat << EOF
RsOJ Admin Management Tool

Usage:
  $0 add <username> <role>     - Grant admin role to user
  $0 remove <username>         - Remove admin privileges
  $0 list                      - List all administrators

Admin Roles:
  problem_admin     - Dashboard and problem management
  community_admin   - Dashboard only; moderation APIs are not implemented yet
  super_admin       - All currently implemented control-panel APIs

Examples:
  $0 add alice super_admin
  $0 add bob problem_admin
  $0 add charlie community_admin
  $0 remove alice
  $0 list

Permissions:
  ┌─────────────────┬──────────┬───────────┬──────────────┐
  │ Feature         │ Problem  │ Community │ Super Admin  │
  ├─────────────────┼──────────┼───────────┼──────────────┤
  │ Dashboard       │ ✓        │ ✓         │ ✓            │
  │ Problems        │ ✓        │           │ ✓            │
  │ System Monitor  │          │           │ ✓            │
  │ Users           │          │           │ ✓            │
  │ Settings        │          │           │ ✓            │
  └─────────────────┴──────────┴───────────┴──────────────┘
EOF
}

case "$1" in
    add|set|grant)
        set_admin "$2" "$3"
        ;;
    remove|revoke|delete)
        remove_admin "$2"
        ;;
    list|ls)
        list_admins
        ;;
    help|-h|--help)
        show_help
        ;;
    *)
        show_help
        exit 1
        ;;
esac
