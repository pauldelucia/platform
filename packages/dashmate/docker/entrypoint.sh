#!/bin/sh
USER_ID=${LOCAL_UID:?err}
GROUP_ID=${LOCAL_GID:?err}
DOCKER_GROUP_ID=$(stat -c %g /var/run/docker.sock)
USERNAME=dashmate
GROUP=docker

# check if user with our uid exists in the system
if [ ! $(getent passwd $USER_ID | grep $USER_ID -q) ]; then
  adduser -u $USER_ID -D -H $USERNAME
else
  USERNAME=$(getent passwd $USER_ID | cut -d: -f1)
fi

# check if docker group exists in the container
if [ -z $(getent group $DOCKER_GROUP_ID) ] ; then
  addgroup -g $DOCKER_GROUP_ID $GROUP
else
  GROUP=$(getent group $DOCKER_GROUP_ID | cut -d: -f1)
fi

# check if our user belongs to docker group
if [ ! $(id -nG $USERNAME | grep -q $GROUP) ]; then
  adduser $USERNAME $GROUP
fi

su $USERNAME

exec "$@"
