# Use an official Node runtime as the base image
FROM node:20 as build-stage

# Set the working directory in the container
WORKDIR /usr/src/app

# Copy package.json and package-lock.json for efficient caching
COPY package*.json ./

# Install the application's dependencies including dev dependencies
RUN npm install

# Copy the rest of the application's files into the container
COPY . .

# Compile TypeScript to JavaScript
RUN npm run build

# Use another stage for the production image to keep it lean
FROM node:20-slim

# Set the working directory in the container
WORKDIR /usr/src/app

# Copy the relevant built files from the previous stage
COPY --from=build-stage /usr/src/app/dist ./dist
COPY package*.json ./

# Install only production dependencies
RUN npm install --only=production

# Start the Node.js application
CMD [ "node", "dist/index.js" ]

# Expose port 3000 for the Express server to be accessible outside the container
EXPOSE 3001
