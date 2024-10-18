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
	expected_version="$LAST_TAG-pr-$PR_NUMBER"
	echo "Checking if there is already a tag"
	RAW=$(curl -H "Authorization: Bearer $GH_TOKEN" -s --fail "https://ghcr.io/v2/$REPOSITORY_NAME/$IMAGE_NAME/tags/list")
	if [ $? -ne 0 ]; then
		echo "No tags found defaulting to 1"
		VERSION="$expected_version.1"
		echo "Got version: $VERSION"
		echo "VERSION=$VERSION" >> $GITHUB_OUTPUT
		exit 0
	fi

	TAGS=$( $RAW | jq -r '.tags[]')

	FILTERED_TAGS=$(echo "$TAGS" | grep -E "^$expected_version.[0-9]+$")
	
	# Check if any filtered tags were found
	if [ -z "$FILTERED_TAGS" ]; then
	 # Default to 1 if no matching tags were found
	 NEW_VERSION=1
	else
		# Find the latest tag with the highest version number
		LATEST_TAG=$(echo "$FILTERED_TAGS" | sort -V | tail -n 1)
		# Extract the version number from the latest tag
		number=$(echo $LATEST_TAG | sed "s/^$expected_version.\([0-9]*\)$/\1/")

		# Increment the version number
		NEW_VERSION=$((number + 1))
	fi
	 VERSION="$expected_version.$NEW_VERSION"
	 echo "Got version: $VERSION"
	 echo "VERSION=$VERSION" >> $GITHUB_OUTPUT
fi

