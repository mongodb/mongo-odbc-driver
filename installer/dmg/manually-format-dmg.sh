#!/usr/bin/env bash

#WARNING: THIS SCRIPT WILL NOT WORK ON EVERGREEN

#This script is for formatting the .DS_Store for our installer. It easily allows
#for changing the layout based on pixels, as can be seen below, versus having to
#format with clicking and dragging. Unfortunately, we cannot run this on evergreen
#because it still requires a gui to operate, but we can use this manually on our
#own macos computers in order to format the .DS_Store.
if [ ! -d create-dmg ]; then
    git clone https://github.com/create-dmg/create-dmg.git
fi

PRODUCT=mongoodbc.pkg

./create-dmg/create-dmg --volname mongoodbc\
	--background resources/background.png\
	--window-pos 200 120\
	--window-size 600 480\
	--icon-size 100\
	--icon "$PRODUCT" 200 190 \
	mongoodbc.dmg\
	dmg-contents

#Copy the created contents to our dmg-contents directory. Make sure to check these
#in.

#First we mount the dmg
hdiutil attach mongoodbc.dmg

#Next, copy the important gui formatting
cp /Volumes/mongoodbc/.DS_Store dmg-contents/
cp -R /Volumes/mongoodbc/.background dmg-contents/

#Now go ahead and unmount the dmg
hdiutil detach /dev/disk2s1

