#!/bin/bash

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

cd ${CI_PROJECT_DIR}/daemon
RUST_LOG=error ${CI_PROJECT_DIR}/target/release/api-daemon &
export RUST_LOG=debug
export RUST_BACKTRACE=1

cd ${CI_PROJECT_DIR}/services/apps/appscmd/tests

# Let the daemon start and initialize.
sleep 5

CMD="${CI_PROJECT_DIR}/target/release/appscmd --socket /tmp/apps_service_uds.sock"
FIXTURES="${CI_PROJECT_DIR}/services/apps/test-fixtures"
VROOT="${CI_PROJECT_DIR}/prebuilts/http_root/webapps/vroot"

${CMD} --json list > apps_observed.json
md5sum apps_expected.json | sed s/expected/observed/ | md5sum -c

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
	md5sum apps_expected_${expect}.json | sed s/expected_${expect}/observed/ | md5sum -c

	# verify the checksum of the application.zip
	origin=`md5sum ${FIXTURES}/${from} | cut -d\  -f1`
	install=`md5sum ${VROOT}/${expect}/application.zip | cut -d\  -f1`
	[ "${origin}" = "${install}" ] || exit 2
done
