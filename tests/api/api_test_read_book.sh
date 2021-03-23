#!/bin/bash

. tests/api/_header.sh

# create book
curl -H "Content-Type: application/json" --request POST --data '{"market":"AUDETH"}' $HOST:$PORT/book > /dev/null 2> /dev/null

OUTPUT=$(curl -H "Accept: application/json" $HOST:$PORT/book/AUDETH 2> /dev/null)

if [ "$OUTPUT" != '{"market":"AUDETH","bids":{},"asks":{},"ltp":0,"depth":[0,0],"crossed":false,"spread":0}' ]; then
    echo "[FAIL] read_book: " $OUTPUT
    exit 1 # FAIL
fi

. tests/api/_footer.sh

