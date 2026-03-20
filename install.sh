#!/bin/sh

INSTALL_PREFIX=${INSTALL_PREFIX:-""}
INSTALL=${INSTALL:-"install"}
INSTALL_OWNER=${INSTALL_OWNER:-"-o root -g root"}
DESTDIR=${DESTDIR:-"/usr/local"}

if [ "${DESTDIR}" = "/" ]; then
  BINDIR="/usr/bin"
  ETCDIR="/etc"
  MAN_DIR="/usr/share/man"
else
  BINDIR="${DESTDIR}/bin"
  ETCDIR="${DESTDIR}/etc"
  MAN_DIR="${DESTDIR}/share/man"
fi

echo "Installing binary"
"${INSTALL}" -d -m 755 ${INSTALL_OWNER} "${INSTALL_PREFIX}${BINDIR}" | exit 1
"${INSTALL}" -m 755 ${INSTALL_OWNER} target/release/cratelist "${INSTALL_PREFIX}${BINDIR}" | exit 1

echo "Installation successful"
