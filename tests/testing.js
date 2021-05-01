// A simple testing framework for SIDL defined interfaces.

"use strict";

function deep_equals(a, b) {
  // If they're triple equals, then it must be equals!
  if (a === b) {
    return true;
  }

  // If they weren't equal, they must be objects to be different
  if (typeof a != "object" || typeof b != "object") {
    return false;
  }

  // But null objects won't have properties to compare
  if (a === null || b === null) {
    return false;
  }

  // Make sure all of a's keys have a matching value in b
  for (let k in a) {
    if (!deep_equals(a[k], b[k])) {
      return false;
    }
  }

  // Do the same for b's keys but skip those that we already checked
  for (let k in b) {
    if (!(k in a) && !deep_equals(a[k], b[k])) {
      return false;
    }
  }

  return true;
}

class SidlEventHandler {
  constructor(target, event_kind) {
    this.values = [];
    this.waiters = [];
    this.handler = this.handle_event.bind(this);
    this.target = target;
    this.event_kind = event_kind;
    this.target.addEventListener(event_kind, this.handler);
  }

  handle_event(value) {
    // We got a new event value. Push it to our queue, and check
    // if we have waiters for it.
    this.values.push(value);
    this.notify_next_waiter();
  }

  notify_next_waiter() {
    if (!this.waiters.length || !this.values.length) {
      return;
    }

    this.waiters.shift()(this.values.shift());
  }

  add_waiter(func) {
    this.waiters.push(func);
    this.notify_next_waiter();
  }

  stop() {
    this.target.removeEventListener(this.event_kind, this.handler);
  }
}

class ServiceTester {
  constructor(service, tester_name, session) {
    this.service = service;
    this.tester_name = tester_name || "";
    this.results = [];
    this.session = session;
  }

  // Returns a handler to wait on an event to be dispatched.
  setup_event(event_kind) {
    return this.setup_event_on(this.service, event_kind);
  }

  // Returns a handler to wait on an event to be dispatched.
  setup_event_on(object, event_kind) {
    return new SidlEventHandler(object, event_kind);
  }

  // Return a promise than resolves with the next value dispatched
  // for this event.
  next_event_value(event_handler) {
    return new Promise(resolve => {
      event_handler.add_waiter(resolve);
    });
  }

  assert_event_eq(description, event_handler, expected = undefined) {
    return this.assert_eq(
      description,
      () => {
        return this.next_event_value(event_handler);
      },
      expected
    );
  }

  // Runs an async runnable and checks the returned value.
  assert_eq(description, runnable, expected, transform) {
    let start = Date.now();
    description = `${this.tester_name}:${description}`;
    return new Promise(resolve => {
      try {
        runnable(this.service).then(
          observed => {
            let elapsed = Date.now() - start;
            if (transform) {
              observed = transform(observed);
            }
            if (deep_equals(observed, expected)) {
              this.results.push({ description, success: true, elapsed });
            } else {
              this.results.push({
                description,
                success: false,
                observed: `${typeof observed} ${JSON.stringify(observed)}`,
                expected: `${typeof expected} ${JSON.stringify(expected)}`
              });
            }
            resolve();
          },
          error => {
            this.results.push({ description, success: false, error });
            resolve();
          }
        );
      } catch(error) {
        this.results.push({ description, success: false, error});
        resolve();
      }
    });
  }

  // Asserts the rejected value of a call instead of the resolved one.
  assert_rej_eq(description, runnable, expected, transform) {
    let start = Date.now();
    description = `${this.tester_name}:${description}`;
    return new Promise(resolve => {
      try {
        runnable(this.service).then(
          error => {
            this.results.push({ description, success: false, error });
            resolve();
          },
          observed => {
            observed = observed.value;
            let elapsed = Date.now() - start;
            if (transform) {
              observed = transform(observed);
            }
            if (deep_equals(observed, expected)) {
              this.results.push({ description, success: true, elapsed });
            } else {
              this.results.push({
                description,
                success: false,
                observed: `${typeof observed} ${JSON.stringify(observed)}`,
                expected: `${typeof expected} ${JSON.stringify(expected)}`
              });
            }
            resolve();
          }
        );
      } catch(error) {
        this.results.push({ description, success: false, error});
        resolve();
      }
    });
  }
}

// Returns a promise resolving to a ServiceTester attached to the service.
function test_service(service, tester_name, existing_session) {
  return new Promise((resolve, reject) => {
    let session = existing_session ? existing_session : new lib_session.Session();
    let sessionstate = {
      onsessionconnected() {
        service.get(session).then(service => {
          resolve(new ServiceTester(service, tester_name, session));
        }, reject);
      },

      onsessiondisconnected() {
        reject("Session Disconnected");
      }
    };

    session.open("websocket", "localhost:8081", "secrettoken", sessionstate);
  });
}

// Aggregates the results of several testers and combine them to produce
// a single result set.
class TestReporter {
  constructor(testers) {
    this.results = [];
    testers.forEach(tester => {
      this.results = this.results.concat(tester.results);
    });
  }

  // Creates a test report in an element with id "test-results".
  output() {
    let element = document.createElement("div");
    element.setAttribute("id", "test-results");

    let success = 0;
    let failures = 0;
    let html = `<div id="header">${this.results.length} tests completed.</div>`;
    this.results.forEach(item => {
      if (item.success) {
        success += 1;
        html += `<div class="success">[${item.description}] : Success</div>`;
      } else {
        failures += 1;
        html += `<div class="failure">[${item.description}] : Failure
        <div>Expected: <pre>${item.expected}</pre></div>
        <div>Observed: <pre>${item.observed}</pre></div>
        </div>`;
      }
    });

    html += `<div id="footer" class="${(failures == 0) ? 'success' : ''}"><span class="success">Success: ${success}</span>, <span class="failure">Failures: ${failures}</span></div>`;

    element.innerHTML = html;
    let hidden = document.createElement("div");
    hidden.classList.add("json");
    hidden.textContent = JSON.stringify(this.results);
    document.body.appendChild(element);
    document.body.appendChild(hidden);
  }
}

function createAsyncTask() {
  const asyncTask = {};
  asyncTask.isFinished = new Promise((resolve) => {
    asyncTask.finish = resolve;
  });;
  return asyncTask;
}
