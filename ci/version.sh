if [ -z "$TRAVIS_TAG" ]
then
    HASH=$(git log --pretty=format:'%h' -n 1)
    sed -E "s/version = \"(.+)\"$/version = \"\1-$HASH\"/" Cargo.toml > Cargo.toml.versioned && mv Cargo.toml.versioned Cargo.toml
fi
