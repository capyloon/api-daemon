
## Build and Update for Test Page

> $ BUILD_TYPE=debug ./update.sh


## Test Page

$SIDL/service/procmanager/tests/test.html is a test page to add a process to
the foreground.  To try it, you need to forward your local TCP 8081
port to 80 port of the device.

> $ adb forward tcp:8081 tcp:80

> $ adb shell api-daemon.sh

Then, you just use a browser to visit the test.html page.

## How to Use The API in The System APP?

First of all, System APP should know PID of content processes.
Gecko should provide a way to enable that.
Let's assume there is a way to do that.

All new content processes start in the same group that the fork server
process is in.  Let's assume the fork server process is in try_to_keep
group, then all new content processes are in try_to_keep group too.

### Switching APPs

Assuming there are two APPs, A1 is the foreground APP, and A2 is a
background APP.  When the user switches from A1 to A2.  A1 should be
sent to background and A2 should be brought to the foreground.  The
system APP should do something like this.

```
    // Switch A2 to the foreground
    procmanager.begin("sysapp").then(ok => {
        return procmanager.add(pid_A2, lib_procmanager.GroupType.FOREGROUND);
    }).then(ok => {
        return procmanager.add(pid_A1, lib_procmanager.GroupType.BACKGROUND);
    }).then(ok => {
        return procmanager.commit();
    }).then(ok => {
        console.log("Now A2 is in the foreground!");
    });
```

### Important background APPs

Some background APPs are more important than others.  They should not
be killed if possible.  For example, you don't want to kill music
player in the background if possible.

For example, the music player is being switched to the background.
Then, the system APP probably should do following instructions.

```
    procmanager.begin("sysapp").then(ok => {
        return procmanager.add(pid_music_player, lib_procmanager.GroupType.TRY_TO_KEEP);
    }).then(ok => {
        return procmanager.add(pid_new_foreground, lib_procmanager.GroupType.FOREGROUND);
    }).then(ok => {
        return procmanager.commit();
    }).then(ok => {
        console.log("Now the music player is in the foreground!");
    });
```

### Kill An APP

If user kill an APP, the system APP should remove it from the API,
although it is not necessary for cgroup, the backend of this API.
Assuming A1 is killed by the user, and A2 is going to be the new
foreground.

```
    procmanager.begin("sysapp").then(ok => {
        return procmanager.add(pid_A2, lib_procmanager.GroupType.FOREGROUND);
    }).then(ok => {
        return procmanager.remove(pid_music_player);
    }).then(ok => {
        return procmanager.commit();
    }).then(ok => {
        console.log("Now A1 has been removed!");
    });
```
