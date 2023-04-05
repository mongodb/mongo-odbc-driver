#!/bin/sh

rm ./*.dmg || true
rm dmg-contents/*.pkg || true

VERSION=$1
ROOT="/Library/MongoDB/MongoDB Atlas SQL ODBC/$VERSION"
rm -Rf components
mkdir -p components/"$ROOT"
mkdir -p scripts
cp ./libatsql.dylib components/"$ROOT"/
cp ./macos_postinstall scripts/postinstall
cp ./resources/*.rtf components/"$ROOT"/
cp ../../README.md components/"$ROOT"/

# build component pkg
pkgbuild --root=components/ --scripts=scripts/ --identifier='MongoDB Atlas SQL ODBC' 'mongoodbc-component.pkg'

# set the version based on $VERSION
sed -i '.bak' "s|__VERSION__|$VERSION|g" distribution.xml

PRODUCT=mongoodbc.pkg
# build product pkg (which can install multiple component pkgs, but we only have one)
productbuild --distribution distribution.xml \
	--resources ./resources \
	--package-path . \
	"$PRODUCT"

mv "$PRODUCT" dmg-contents/

hdiutil create -fs HFS+ -srcfolder dmg-contents -volname mongoodbc mongoodbc.dmg
