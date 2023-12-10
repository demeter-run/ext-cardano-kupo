#!/bin/bash

# Check the command passed to the Docker container
case "$1" in
    backup)
        /app/backup.sh
        ;;
    restore)
        /app/restore.sh
        ;;
    *)
        echo "Usage: {backup|restore}"
        exit 1
        ;;
esac
