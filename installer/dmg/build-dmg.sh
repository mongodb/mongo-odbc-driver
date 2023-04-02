#!/bin/sh

rm ./*.dmg || true
rm dmg-contents/*.pkg || true

VERSION=$1
ROOT="/Library/MongoDB/AtlasSQL ODBC/$VERSION"
mkdir -p components/"$ROOT"
mv ./libatsql.dylib components/"$ROOT"/
cp ./resources/*.rtf components/"$ROOT"/
cp ../../README.md components/"$ROOT"/

# build component pkg
pkgbuild --root=components/ --scripts=scripts/ --identifier='AtlasSQL ODBC' 'mongoodbc-component.pkg'

# set the version based on $VERSION
sed -i '.bak' "s|__VERSION__|$VERSION|g" distribution.xml

PRODUCT=mongoodbc.pkg
# build product pkg (which can install multiple component pkgs, but we only have one)
productbuild --distribution distribution.xml \
	--resources ./resources \
	--package-path . \
	"$PRODUCT"

mkdir -p dmg-contents

mv "$PRODUCT" dmg-contents/

if [ ! -d create-dmg ]; then
    git clone https://github.com/create-dmg/create-dmg.git
fi

./create-dmg/create-dmg --volname mongoodbc\
	--background resources/background.png\
	--window-pos 200 120\
	--window-size 600 480\
	--icon-size 100\
	--icon "$PRODUCT" 200 190 \
	mongoodbc.dmg\
	dmg-contents
