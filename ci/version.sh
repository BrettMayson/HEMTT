if [ -z "$TRAVIS_TAG" ]
then
    HASH=$(git log --pretty=format:'%h' -n 1)
    sed -i -E "s/version = \"([^\"]+?)\"$/version = \"\1-$HASH\"/" Cargo.toml
fi
