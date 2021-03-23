#!/bin/bash

. tests/api/_header.sh

OUTPUT=$(curl -H "Content-Type: application/json" --request POST --data '{"market":"AUDETH"}' $HOST:$PORT/book 2> /dev/null)


if [ "$OUTPUT" != "Created new market" ]; then
    echo "[FAIL] create_book: " $OUTPUT
    exit 1 # FAIL
fi

. tests/api/_footer.sh

