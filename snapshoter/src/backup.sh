#!/bin/bash

# Load Environment Variables
export AWS_ACCESS_KEY_ID=your_access_key_id
export AWS_SECRET_ACCESS_KEY=your_secret_access_key
S3_BUCKET=your_s3_bucket_name
S3_FILE_URI=your_s3_file_uri
SOURCE_FOLDER=your_source_folder

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
