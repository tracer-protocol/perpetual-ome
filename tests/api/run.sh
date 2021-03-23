#!/bin/bash

. tests/api/_header.sh

RETVAL=0

for api_test in tests/api/api_test_*.sh
do
    . $api_test $HOST $PORT

    if [ "$?" -ne 0 ]; then
        echo "[FAIL] " $api_test
        RETVAL=1
    else
        echo "[PASS] " $api_test
    fi    
done

exit $RETVAL

