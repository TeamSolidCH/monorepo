push_string="push"
if [[ "$GITHUB_EVENT_NAME" == "$push_string" ]]; then
	echo "Run for tag"
	tag_name=$GITHUB_REF_NAME
	echo "Tag name: $tag_name"
	VERSION=$(echo $tag_name | rev | cut -d'/' -f1 | rev)
	echo "Got version: $VERSION"
	echo "VERSION=$VERSION" >> $GITHUB_OUTPUT
else
	echo "Run for pull_request" 
	echo "Got pre-release: pr-$PR_NUMBER"
	last_tag=$(git describe --tags --abbrev=0 --match="calendarbot/*")
	if [ -z "$last_tag" ]; then
		echo "No tags found defaulting to calendarbot/v0.0.1"
		last_tag="calendarbot/v0.0.1"
	fi
	VERSION="$(echo $last_tag | rev | cut -d'/' -f1 | rev)-pr-$PR_NUMBER"
	echo "Got version: $VERSION"
	echo "VERSION=$VERSION" >> $GITHUB_OUTPUT
fi

