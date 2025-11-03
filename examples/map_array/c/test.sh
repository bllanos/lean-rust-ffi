set -ex

# Assuming Lake is already in the executable search path
LAKE=lake

./clean.sh

LAKE=$LAKE make run
