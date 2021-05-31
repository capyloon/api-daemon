#!/bin/bash

# setup jq to verify json output
if [ -z "$(which jq)" ]; then
	apt update && apt install -y jq
fi

function compare_list() {
	jq --argfile a $1 --argfile b $2 \
	-n '($a | (.. | arrays) |= sort) as $a | ($b | (.. | arrays) |= sort) as $b | $a == $b'
}


set -e

if [ -z ${CI_PROJECT_DIR+x} ];
then
    echo "Please set CI_PROJECT_DIR to the path of your SIDL repository.";
    exit 1;
fi

# Kill child processes on exit.
trap 'jobs -p | xargs kill' EXIT

# Reset apps
rm -rf $CI_PROJECT_DIR/prebuilts/http_root/webapps/

# Align with config-webdriver.toml
rm -rf $CI_PROJECT_DIR/tests/webapps
ln -s $CI_PROJECT_DIR/services/apps/test-fixtures/webapps $CI_PROJECT_DIR/tests/webapps

DONT_CREATE_WEBAPPS=1 $CI_PROJECT_DIR/tests/webdriver.sh $CI_PROJECT_DIR/services/apps/client/test/dummy.html > /dev/null 2>&1 &

cd ${CI_PROJECT_DIR}/services/apps/appscmd/tests

# Let the daemon start and initialize.
sleep 5

CMD="${CI_PROJECT_DIR}/target/release/appscmd --socket /tmp/apps_service_uds.sock"
FIXTURES="${CI_PROJECT_DIR}/services/apps/test-fixtures"
VROOT="${CI_PROJECT_DIR}/prebuilts/http_root/webapps/vroot"

${CMD} --json list > apps_observed.json
result=`compare_list apps_expected.json  apps_observed.json`
[ "${result}" = "true" ] || exit 2

# array of test cases
# "expected_name  application_to_install"
tests=(
"gallery     webapps/gallery/application.zip"
"gallery     webapps/gallery1/application.zip"
"calculator1 webapps/calculator/application.zip"
"calculator1 webapps/calculator1/application.zip"
"helloworld  apps-from/helloworld/application.zip"
"helloworld  apps-from/helloworld1/application.zip"
"12345       apps-from/12345/application.zip"
)

echo tests: ${tests}
for (( i=0; i<${#tests[@]}; i++ ));
do
	test=${tests[$i]}
	echo "test: ${test}"
	[ -z "${test}" ] && continue
	from=`echo ${test}  | cut -d\  -f2`
	expect=`echo ${test}  | cut -d\  -f1`

	# Will install with a new unique name if it is not allowed to override.
	${CMD} install ${FIXTURES}/${from}
	${CMD} --json list > apps_observed.json

	# verify app list
	result=`compare_list apps_expected_${expect}.json  apps_observed.json`
	[ "${result}" = "true" ] || exit 2

	# verify the checksum of the application.zip
	origin=`md5sum ${FIXTURES}/${from} | cut -d\  -f1`
	install=`md5sum ${VROOT}/${expect}/application.zip | cut -d\  -f1`
	[ "${origin}" = "${install}" ] || exit 2
done

# array of uninstall test cases
# "expected_name  uninstall-manifest_url"
uninstalls=(
"12345          http://launcher.localhost:8081/manifest.webmanifest"
"helloworld     http://12345.localhost:8081/manifest.webmanifest"
)

echo uninstalls: ${uninstalls}
for (( i=0; i<${#uninstalls[@]}; i++ ));
do
	test=${uninstalls[$i]}
	echo "test: ${test}"
	[ -z "${test}" ] && continue
	manifest_url=`echo ${test}  | cut -d\  -f2`
	expect=`echo ${test}  | cut -d\  -f1`

	# Uninstall an app and then get the applist.
	${CMD} uninstall ${manifest_url}
	${CMD} --json list > apps_observed.json

	# verify app list
	result=`compare_list apps_expected_${expect}.json  apps_observed.json`
	[ "${result}" = "true" ] || exit 2
done
