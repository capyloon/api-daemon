// Test driver that provides the capability to run
// one or several SIDL test and automatically wrap it with
// a WebDriver layer.
// Both geckodriver and Firefox need to be in the $PATH
// and the env variable TEST_FIREFOX_PROFILE needs to be
// set to the path of the profile to use.

use fantoccini::error::CmdError;
use fantoccini::{Client, Locator};
use log::{error, info};
use serde::Deserialize;
use std::env;
use std::net::TcpStream;
use std::process::{exit, Command};
use yansi::Paint;

#[derive(Deserialize)]
struct TestData {
    description: String,
    success: bool,
    expected: Option<String>,
    observed: Option<String>,
    error: Option<String>,
    elapsed: Option<u32>,
}

async fn test_script(script: &str, client: &mut Client) -> Result<(), CmdError> {
    println!(
        "{}",
        Paint::blue(format!("WebDriver test of {}", script)).bold()
    );

    client.goto(&script).await?;
    println!("{}", Paint::blue(format!("Navigated to {}", script)).bold());

    let mut element = loop {
        match client.find(Locator::Css(".json")).await {
            Ok(element) => break element,
            Err(_) => {
                println!("{}", Paint::white("Waiting for .json to be available..."));
                ::std::thread::sleep(::std::time::Duration::from_secs(2));
            }
        }
    };

    println!("{}", Paint::blue("Get json element").bold());
    let json = element.text().await?;
    println!("{}", Paint::blue("Get json text").bold());
    println!("{}", json);

    let tests: Vec<TestData> = match serde_json::from_str(&json) {
        Ok(value) => value,
        Err(err) => {
            println!(
                "{} : {}",
                Paint::red("Failed to get test data, error").bold(),
                Paint::white(err)
            );
            exit(4);
        }
    };

    let success = tests.iter().filter(|item| item.success).count() == tests.len();

    for test in tests {
        if test.success {
            println!(
                "{} ({}ms)",
                Paint::green(test.description).bold(),
                test.elapsed.unwrap()
            );
        } else if test.error.is_none() {
            println!(
                "{} : expected {} but got {}",
                Paint::red(test.description).bold(),
                Paint::white(test.expected.unwrap()),
                Paint::white(test.observed.unwrap())
            );
        } else {
            println!(
                "{} : {}",
                Paint::red(test.description).bold(),
                Paint::white(test.error.unwrap())
            );
        }
    }

    if !success {
        println!("{}", Paint::blue("WebDriver test failed").bold());
        exit(4);
    }

    println!("{}", Paint::blue("WebDriver test successful").bold());

    Ok(())
}

fn find_process(process_name: &str) -> bool {
    for prc in procfs::process::all_processes().unwrap() {
        if prc.stat.comm.as_str() == process_name {
            return true;
        }
    }
    false
}

#[tokio::main]
async fn main() -> Result<(), CmdError> {
    env_logger::init();

    info!("Starting SIDL integration tests driver.");

    // Check that we have at least one script parameter
    if env::args().nth(1).is_none() {
        error!(
            "Usage: {} <path_to_test1.html> ... <path_to_testN.html>",
            env::args().next().unwrap()
        );
        exit(1);
    }

    let geckodriver = match env::var("GECKODRIVER_BIN") {
        Ok(value) => value,
        Err(_) => "geckodriver".into(),
    };
    info!("Starting geckodriver");
    let mut geckodriver_handle = match Command::new(geckodriver)
        .arg("--connect-existing")
        .arg("--marionette-port")
        .arg("2828")
        .spawn()
    {
        Ok(handle) => handle,
        Err(err) => {
            error!("Failed to launch geckodriver: ${}", err);
            exit(3);
        }
    };
    info!("geckodriver pid is {}", geckodriver_handle.id());

    let profile_dir = match env::var("TEST_FIREFOX_PROFILE") {
        Ok(value) => value,
        Err(_) => {
            error!("Please set the TEST_FIREFOX_PROFILE environment variable properly.");
            exit(2);
        }
    };

    let firefox = match env::var("FIREFOX_BIN") {
        Ok(value) => value,
        Err(_) => "firefox".into(),
    };

    let headless = match env::var("TEST_NO_HEADLESS") {
        Ok(_) => "--headless__",
        Err(_) => "--headless",
    };
    info!("Starting Firefox with profile at {}", profile_dir);
    let mut firefox_handle = match Command::new(firefox)
        .arg(headless)
        .arg("-no-remote")
        .arg("-marionette")
        .arg("-profile")
        .arg(profile_dir)
        .spawn()
    {
        Ok(handle) => handle,
        Err(err) => {
            error!("Failed to launch Firefox: ${}", err);
            exit(4);
        }
    };
    info!("Firefox pid is {}", firefox_handle.id());

    // geckodriver will wait for up to 60s for Firefox to start, but we need
    // to make sure it is ready to accept connections. That can take more time
    // than expected on CI so we try to connect in a loop first.
    info!("Testing connectivity to geckodriver");
    let mut retries = 0;
    loop {
        // Try to connect to localhost:4444 with a socket,
        // and wait the homescreen app is started.
        if TcpStream::connect("localhost:4444").is_ok() &&
           find_process("Web Content") {
            break;
        }

        if retries < 12 {
            ::std::thread::sleep(::std::time::Duration::from_secs(5));
        } else {
            error!("Failed to start geckodriver properly");
            let _ = geckodriver_handle.kill();
            info!("geckodriver killed");
            let _ = firefox_handle.kill();
            info!("firefox killed");
            exit(5);
        }
        retries += 1;
        info!("Retrying to connect to geckodriver: {}", retries);
    }
    info!("Connectivity to geckodriver verified.");

    let mut client = Client::new("http://localhost:4444")
        .await
        .map_err(|error| unimplemented!("failed to connect to WebDriver: {:?}", error))
        .expect("Failed to connect to WebDriver");

    println!(
        "{}",
        Paint::blue("Client connected to http://localhost:4444").bold()
    );

    for script in env::args().into_iter().skip(1) {
        test_script(&script, &mut client).await?;
    }

    let res = client.close().await;
    println!(
        "{}",
        Paint::blue(format!("Connection closed: {:?}", res)).bold()
    );

    info!("About to shutdown");
    let _ = geckodriver_handle.kill();
    info!("geckodriver killed");
    let _ = firefox_handle.kill();
    info!("firefox killed");

    res
}
