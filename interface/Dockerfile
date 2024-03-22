# Use an official Node runtime as the base image
FROM node:20

# Set the working directory in the container
WORKDIR /usr/src/app

# compile python from source - avoid unsupported library problems
RUN apt update -y && apt upgrade -y && \
    apt-get install python3-pip -y

# Copy package.json, yarn.lock for efficient caching
COPY package.json yarn.lock ./

# Install the application's dependencies using yarn
RUN yarn global add node-gyp && yarn install --frozen-lockfile

# Copy the rest of the application's files into the container
COPY . .

# Build the React application for production
RUN yarn run build

# Use a lightweight version of Node.js as a second stage build
FROM node:20-slim

WORKDIR /usr/src/app

# Copy the build folder from the build image
COPY --from=0 /usr/src/app/build /usr/src/app/build

ENV MODE="production"

# Install serve to serve the React app
RUN npm install -g serve

# Start the React application
CMD [ "serve", "-l", "tcp://0.0.0.0:3002", "-s", "build" ]

# Expose port 5000 for the React app to be accessible outside the container
EXPOSE 3002
