push_string="push"
if [[ "$GITHUB_EVENT_NAME" == "$push_string" ]]; then
	echo "Run for tag"
	tag_name=$GITHUB_REF_NAME
	echo "Tag name: $tag_name"
	VERSION=$(echo $STRING | rev | cut -d'/' -f1 | rev)
	echo "Got version: $VERSION"
	echo "VERSION=$VERSION" >> $GITHUB_OUTPUT
else
	echo "Run for pull_request" 
	echo "Got pre-release: pr-$PR_NUMBER"
	VERSION="$LAST_TAG-pr-$PR_NUMBER"
	echo "Got version: $VERSION"
	echo "VERSION=$VERSION" >> $GITHUB_OUTPUT
fi

