#!/bin/bash

if [ -f .env ]; then
  export $(grep -v '^#' .env | xargs)
fi

DB_HOST=$DIGITALOCEAN_DB_HOST
DB_USER=$DIGITALOCEAN_DB_USER
DB_NAME=$DIGITALOCEAN_DB_NAME
DB_PORT=$DIGITALOCEAN_DB_PORT
DB_PASSWORD=$DIGITALOCEAN_DB_PASSWORD
TIMESTAMP=$(date +"%Y-%d-%A-at-%-I:%M%p")
BACKUP_FILE="./BACKUP-KLASSYMISSY-$TIMESTAMP.sql"

export PGPASSWORD="$DB_PASSWORD"

pg_dump -U "$DB_USER" -h "$DB_HOST" -p "$DB_PORT" -d "$DB_NAME"  -f $BACKUP_FILE

if [ -f "$BACKUP_FILE" ]; then
  echo "$BACKUP_FILE"
else
  echo "Error: Failed to create the database backup."
  if [ -f "$BACKUP_FILE" ]; then
    rm -rf "$BACKUP_FILE"
  fi
  exit 1
fi
