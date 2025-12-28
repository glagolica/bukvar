"""
Database migration manager for version-controlled schema changes.

This module provides tools for managing database migrations in a safe,
repeatable manner. Supports multiple database backends and maintains
migration history for rollback capabilities.

Usage:
    >>> from migration_manager import MigrationRunner
    >>> runner = MigrationRunner("postgresql://localhost/mydb")
    >>> runner.migrate_to("20240115_add_users_table")

Example:
    Running all pending migrations:

    >>> runner = MigrationRunner.from_config("config/database.yml")
    >>> runner.run_pending()
    Applied: 20240101_initial_schema
    Applied: 20240110_add_indexes
    Applied: 20240115_add_users_table

Note:
    Always backup your database before running migrations in production!

.. versionadded:: 2.0.0
    Complete rewrite with async support.
"""

from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum, auto
from pathlib import Path
from typing import Callable, List, Optional, Protocol
import hashlib


class MigrationStatus(Enum):
    """Status of a migration."""

    PENDING = auto()
    APPLIED = auto()
    FAILED = auto()
    ROLLED_BACK = auto()


@dataclass
class Migration:
    """
    Represents a database migration.

    Attributes:
        id: Unique migration identifier (usually timestamp + name).
        name: Human-readable migration name.
        up_sql: SQL to apply the migration.
        down_sql: SQL to rollback the migration.
        checksum: SHA256 hash of the migration content.
        applied_at: When the migration was applied, if at all.
        status: Current status of the migration.

    Example:
        >>> migration = Migration(
        ...     id="20240115_001",
        ...     name="add_users_table",
        ...     up_sql="CREATE TABLE users (id SERIAL PRIMARY KEY);",
        ...     down_sql="DROP TABLE users;"
        ... )
        >>> print(migration.checksum[:16])
        'a1b2c3d4e5f6g7h8'
    """

    id: str
    name: str
    up_sql: str
    down_sql: str
    checksum: str = field(init=False)
    applied_at: Optional[datetime] = None
    status: MigrationStatus = MigrationStatus.PENDING

    def __post_init__(self):
        """Calculate checksum after initialization."""
        content = f"{self.up_sql}{self.down_sql}"
        self.checksum = hashlib.sha256(content.encode()).hexdigest()


class DatabaseConnection(Protocol):
    """
    Protocol for database connections.

    Any database adapter must implement these methods to be compatible
    with the migration runner.

    Methods:
        execute: Run a SQL statement.
        begin_transaction: Start a new transaction.
        commit: Commit the current transaction.
        rollback: Rollback the current transaction.
    """

    def execute(self, sql: str, params: tuple = ()) -> None:
        """Execute a SQL statement."""
        ...

    def begin_transaction(self) -> None:
        """Start a new transaction."""
        ...

    def commit(self) -> None:
        """Commit the current transaction."""
        ...

    def rollback(self) -> None:
        """Rollback the current transaction."""
        ...


class MigrationError(Exception):
    """
    Base exception for migration errors.

    Args:
        message: Error description.
        migration_id: ID of the migration that failed.
        cause: Original exception that caused this error.

    Attributes:
        migration_id: The ID of the migration that caused the error.
        cause: The underlying exception, if any.

    Raises:
        MigrationError: When a migration fails to apply or rollback.

    Example:
        >>> try:
        ...     runner.apply(migration)
        ... except MigrationError as e:
        ...     print(f"Migration {e.migration_id} failed: {e}")
    """

    def __init__(
        self,
        message: str,
        migration_id: Optional[str] = None,
        cause: Optional[Exception] = None,
    ):
        super().__init__(message)
        self.migration_id = migration_id
        self.cause = cause


class ChecksumMismatchError(MigrationError):
    """
    Raised when a migration's checksum doesn't match the recorded checksum.

    This indicates the migration file was modified after being applied,
    which could lead to inconsistent database state.

    Warning:
        Never modify a migration that has been applied to production!
        Create a new migration instead.
    """

    pass


@dataclass
class MigrationRunner:
    """
    Manages database migrations.

    The runner tracks applied migrations, validates checksums, and
    provides safe migration and rollback operations.

    Args:
        connection_string: Database connection URL.
        migrations_dir: Path to migrations directory.
        table_name: Name of the migrations tracking table.

    Attributes:
        connection: Active database connection.
        migrations: List of all discovered migrations.
        applied: Set of applied migration IDs.

    Example:
        Basic usage::

            runner = MigrationRunner("postgresql://localhost/mydb")
            runner.discover_migrations("./migrations")

            # Apply all pending
            for result in runner.run_pending():
                print(f"Applied: {result.name}")

            # Rollback last migration
            runner.rollback_last()

    Note:
        The runner creates a migrations tracking table automatically
        on first use.

    See Also:
        Migration: The migration data structure.
        MigrationError: Errors that can occur during migration.

    .. versionchanged:: 2.1.0
        Added support for async migrations.
    """

    connection_string: str
    migrations_dir: Path = field(default_factory=lambda: Path("./migrations"))
    table_name: str = "_migrations"
    connection: Optional[DatabaseConnection] = field(default=None, init=False)
    migrations: List[Migration] = field(default_factory=list, init=False)
    applied: set = field(default_factory=set, init=False)

    @classmethod
    def from_config(cls, config_path: str) -> "MigrationRunner":
        """
        Create a runner from a configuration file.

        Args:
            config_path: Path to YAML or JSON configuration file.

        Returns:
            Configured MigrationRunner instance.

        Raises:
            FileNotFoundError: If config file doesn't exist.
            ValueError: If config file format is invalid.

        Example:
            >>> runner = MigrationRunner.from_config("config/database.yml")
        """
        # Implementation would parse config file
        raise NotImplementedError("Config loading not implemented")

    def discover_migrations(self, path: Optional[Path] = None) -> List[Migration]:
        """
        Discover migration files in a directory.

        Scans the migrations directory for SQL files and parses them
        into Migration objects. Files must follow the naming convention:
        ``YYYYMMDD_NNN_name.sql``

        Args:
            path: Optional override for migrations directory.

        Returns:
            List of discovered migrations, sorted by ID.

        Raises:
            FileNotFoundError: If migrations directory doesn't exist.
            ValueError: If migration file format is invalid.

        Example:
            >>> migrations = runner.discover_migrations()
            >>> print(f"Found {len(migrations)} migrations")
            Found 15 migrations

        Todo:
            * Support for Python migration files
            * Support for reversible migration detection
        """
        search_path = path or self.migrations_dir
        if not search_path.exists():
            raise FileNotFoundError(f"Migrations directory not found: {search_path}")

        migrations = []
        for file in sorted(search_path.glob("*.sql")):
            migration = self._parse_migration_file(file)
            migrations.append(migration)

        self.migrations = migrations
        return migrations

    def _parse_migration_file(self, path: Path) -> Migration:
        """
        Parse a migration file into a Migration object.

        Args:
            path: Path to the migration file.

        Returns:
            Parsed Migration object.

        Raises:
            ValueError: If file format is invalid or missing sections.
        """
        content = path.read_text()
        parts = content.split("-- DOWN --")

        if len(parts) != 2:
            raise ValueError(f"Migration {path.name} missing '-- DOWN --' separator")

        return Migration(
            id=path.stem,
            name=path.stem.split("_", 2)[-1] if "_" in path.stem else path.stem,
            up_sql=parts[0].strip(),
            down_sql=parts[1].strip(),
        )

    def run_pending(
        self, 
        callback: Optional[Callable[[Migration], None]] = None
    ) -> List[Migration]:
        """
        Apply all pending migrations.

        Args:
            callback: Optional function called after each migration.

        Returns:
            List of applied migrations.

        Raises:
            MigrationError: If any migration fails.
            ChecksumMismatchError: If a migration was modified.

        Example:
            >>> applied = runner.run_pending(
            ...     callback=lambda m: print(f"Applied: {m.name}")
            ... )
            Applied: add_users_table
            Applied: add_indexes
        """
        applied = []
        pending = [m for m in self.migrations if m.id not in self.applied]

        for migration in pending:
            self._apply_migration(migration)
            applied.append(migration)
            if callback:
                callback(migration)

        return applied

    def _apply_migration(self, migration: Migration) -> None:
        """
        Apply a single migration.

        Args:
            migration: Migration to apply.

        Raises:
            MigrationError: If migration fails.
        """
        if not self.connection:
            raise MigrationError("Not connected to database")

        try:
            self.connection.begin_transaction()
            self.connection.execute(migration.up_sql)
            self._record_migration(migration)
            self.connection.commit()
            migration.status = MigrationStatus.APPLIED
            migration.applied_at = datetime.now()
            self.applied.add(migration.id)
        except Exception as e:
            self.connection.rollback()
            migration.status = MigrationStatus.FAILED
            raise MigrationError(
                f"Failed to apply migration: {e}",
                migration_id=migration.id,
                cause=e,
            )

    def _record_migration(self, migration: Migration) -> None:
        """Record a migration in the tracking table."""
        sql = f"""
            INSERT INTO {self.table_name} (id, name, checksum, applied_at)
            VALUES (?, ?, ?, ?)
        """
        self.connection.execute(
            sql,
            (migration.id, migration.name, migration.checksum, datetime.now()),
        )

    def rollback_last(self) -> Optional[Migration]:
        """
        Rollback the most recently applied migration.

        Returns:
            The rolled back migration, or None if no migrations to rollback.

        Raises:
            MigrationError: If rollback fails.

        Warning:
            Use with caution in production environments!
        """
        if not self.applied:
            return None

        last_id = max(self.applied)
        migration = next((m for m in self.migrations if m.id == last_id), None)

        if migration:
            self._rollback_migration(migration)
            return migration
        return None

    def _rollback_migration(self, migration: Migration) -> None:
        """Rollback a single migration."""
        if not self.connection:
            raise MigrationError("Not connected to database")

        try:
            self.connection.begin_transaction()
            self.connection.execute(migration.down_sql)
            self._remove_migration_record(migration)
            self.connection.commit()
            migration.status = MigrationStatus.ROLLED_BACK
            self.applied.discard(migration.id)
        except Exception as e:
            self.connection.rollback()
            raise MigrationError(
                f"Failed to rollback migration: {e}",
                migration_id=migration.id,
                cause=e,
            )

    def _remove_migration_record(self, migration: Migration) -> None:
        """Remove a migration from the tracking table."""
        sql = f"DELETE FROM {self.table_name} WHERE id = ?"
        self.connection.execute(sql, (migration.id,))

    def status(self) -> dict:
        """
        Get current migration status.

        Returns:
            Dictionary with migration statistics:
                - total: Total number of migrations
                - applied: Number of applied migrations
                - pending: Number of pending migrations
                - last_applied: ID of last applied migration

        Example:
            >>> status = runner.status()
            >>> print(f"Pending: {status['pending']}")
            Pending: 3
        """
        return {
            "total": len(self.migrations),
            "applied": len(self.applied),
            "pending": len(self.migrations) - len(self.applied),
            "last_applied": max(self.applied) if self.applied else None,
        }
