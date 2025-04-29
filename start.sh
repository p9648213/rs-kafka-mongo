#!/bin/bash
# start.sh

# Start the Kafka consumer in background
./event_consumer &

# Start the API server in foreground
./rs-kafka-mongo