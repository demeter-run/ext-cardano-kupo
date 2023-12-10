# Backup and Restore helper 

## BACKUP Env vars

```
AWS_ACCESS_KEY_ID
AWS_SECRET_ACCESS_KEY
S3_BUCKET
S3_FILE_URI
SOURCE_FOLDER
```

## RESTORE Env vars

```
AWS_ACCESS_KEY_ID
AWS_SECRET_ACCESS_KEY
S3_BUCKET
S3_FILE_URI
DESTINATION_FOLDER
```

## Usage

Set the required env vars and run the docker image with either `backup` or `restore` command