# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    test -f Cargo.lock || cargo generate-lockfile

    cross build --target $TARGET
    cross build --target $TARGET --release

    if [ -z $DISABLE_TESTS ]; then
        cross test --target $TARGET -- --help
        cross test --target $TARGET --release -- --help

        cross run --target $TARGET -- --help
        cross run --target $TARGET --release -- --help
    fi
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
