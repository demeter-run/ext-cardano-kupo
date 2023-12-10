#!/bin/bash

# Load Environment Variables
export AWS_ACCESS_KEY_ID=your_access_key_id
export AWS_SECRET_ACCESS_KEY=your_secret_access_key
S3_BUCKET=your_s3_bucket_name
S3_FILE_URI=your_s3_file_uri
DESTINATION_FOLDER=your_destination_folder

echo "Restoring s3://$S3_BUCKET/$S3_FILE_URI to $DESTINATION_FOLDER"

echo "Making $DESTINATION_FOLDER"
# Create the destination folder if it doesn't exist
mkdir -p $DESTINATION_FOLDER

echo "Downloading s3://$S3_BUCKET/$S3_FILE_URI"
# Download the zip file from S3
aws s3 cp s3://$S3_BUCKET/$S3_FILE_URI $S3_FILE_URI

echo "Unzipping $S3_FILE_URI to $DESTINATION_FOLDER"
# Unzip the file
unzip $S3_FILE_URI -d $DESTINATION_FOLDER

echo "Cleaning up $S3_FILE_URI"
# Clean up the downloaded zip file (optional)
rm $S3_FILE_URI

