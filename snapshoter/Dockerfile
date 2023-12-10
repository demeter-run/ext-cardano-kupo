# Use an official Python base image with Alpine for a small footprint
FROM python:3.8-alpine

# Install AWS CLI and zip utilities
RUN pip install awscli && \
    apk add --no-cache zip unzip bash

# Set the working directory
WORKDIR /app

# Copy the scripts and the entrypoint script into the container
COPY /src/. /app/.

# Make the scripts and entrypoint script executable
RUN chmod +x /app/*.sh

# Set up environment variables (replace with your values or use --build-arg)
# ENV AWS_ACCESS_KEY_ID=your_access_key_id \
#     AWS_SECRET_ACCESS_KEY=your_secret_access_key \
#     S3_BUCKET=your_s3_bucket_name \
#     S3_FILE_URI=your_s3_file_uri \
#     SOURCE_FOLDER=your_source_folder \
#     DESTINATION_FOLDER=your_destination_folder

# Set the entrypoint to the wrapper script
ENTRYPOINT ["/app/entrypoint.sh"]
# CMD ["upload"] # Default action
