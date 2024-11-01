#!/bin/sh

LIBDIRS=`find ./build/dev/chez -type d`
LIBDIRS=`echo $LIBDIRS | tr ' ' ':'`

chez --libdirs "$LIBDIRS" --script "$1"
