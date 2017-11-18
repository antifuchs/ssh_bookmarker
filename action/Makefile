SIGN_ID ?= "Mac Developer: Andreas Fuchs (36K5THSPW7)"

all:
	codesign --force --sign ${SIGN_ID} SSH.lbaction
	zip -r SSH.lbaction.zip SSH.lbaction
