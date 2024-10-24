push_string="push"
pull_request_string="pull_request"
if [[ "$GITHUB_EVENT_NAME" == "$push_string" ]]; then
	if [[ $GITHUB_REF == refs/tags/* ]]; then
		echo "Run for push tag"
		tag_name=$GITHUB_REF_NAME
		echo "Tag name: $tag_name"
		VERSION=$(echo $tag_name | rev | cut -d'/' -f1 | rev)
		echo "Got version: $VERSION"
		tag_latest="true"
		if [[ $VERSION =~ "-" ]]; then
		    echo "Tag is a pre-release"
			tag_latest="false"
        fi
		echo "VERSION=$VERSION" >> $GITHUB_OUTPUT
		echo "TAG_LATEST=$tag_latest" >> $GITHUB_OUTPUT
	elif [[ $GITHUB_REF == refs/heads/main ]]; then
		echo "Run for push to main"
		last_tag=$(git describe --tags --abbrev=0 --match="calendarbot/*")
		if [ -z "$last_tag" ]; then
			echo "No tags found defaulting to calendarbot/v0.0.1"
			last_tag="calendarbot/v0.0.1"
		fi
		VERSION="$(echo $last_tag | rev | cut -d'/' -f1 | rev)-rc.$GITHUB_RUN_NUMBER"
		echo "Got version: $VERSION"
		echo "VERSION=$VERSION" >> $GITHUB_OUTPUT
		echo "TAG_LATEST=false" >> $GITHUB_OUTPUT
	fi
elif [[ "$GITHUB_EVENT_NAME" == "$pull_request_string" ]]; then
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
	echo "TAG_LATEST=false" >> $GITHUB_OUTPUT
else
	echo "No event name found"
	exit 1
fi
