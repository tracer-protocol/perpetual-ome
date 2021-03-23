#!/bin/bash

. tests/api/_header.sh

OUTPUT=$(curl -H "Accept: application/json" $HOST:$PORT/book 2> /dev/null)

if [ "$OUTPUT" != "{}" ]; then
    echo "[FAIL] index_book: " $OUTPUT
    exit 1 # FAIL
fi

. tests/api/_footer.sh

