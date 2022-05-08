#!/bin/bash

if ! command -v npm &> /dev/null
then
    echo "npm could not be found"
    exit
fi

cd frontend

npm i

npm run build

if [ -d "../static" ] ; then
    rm -rf ../static
fi

mv -f ./dist ./static
mv -f ./static ..