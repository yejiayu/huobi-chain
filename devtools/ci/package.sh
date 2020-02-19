#!/bin/bash
set -ev

mkdir package
cp -r ./config ./package/
if [ "$TRAVIS_OS_NAME" == "windows" ]
then
    cp ./target/release/huobi-chain.exe ./package/
    7z a huobi-chain-$TRAVIS_TAG-$TRAVIS_OS_NAME.zip package/
else
    cp ./target/release/huobi-chain ./package/
    tar zcvf huobi-chain-$TRAVIS_TAG-$TRAVIS_OS_NAME.tar.gz package/
fi
