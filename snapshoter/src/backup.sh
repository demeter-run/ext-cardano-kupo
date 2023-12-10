#!/bin/bash

echo "Backing up $SOURCE_FOLDER to s3://$S3_BUCKET/$S3_FILE_URI"

echo "zip -r $ZIP_FILE $SOURCE_FOLDER"
# Zip the Folder
ZIP_FILE="${SOURCE_FOLDER}.zip"
zip -r $ZIP_FILE $SOURCE_FOLDER

echo "Uploading $ZIP_FILE to s3://$S3_BUCKET/$S3_FILE_URI"
# Upload to S3
aws s3 cp $ZIP_FILE s3://$S3_BUCKET/$S3_FILE_URI

echo "Cleaning up $ZIP_FILE"
# Clean up the zip file (optional)
rm $ZIP_FILE
