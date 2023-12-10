#!/bin/bash

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

