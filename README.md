# promkit

[![ci](https://github.com/ynqa/promkit/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/ynqa/promkit/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/promkit)](https://docs.rs/promkit)

A toolkit for building your own interactive prompt in Rust.

## Getting Started

Put the package in your `Cargo.toml`.

```toml
[dependencies]
promkit = "0.10.0"
```

## Features

- Cross-platform support for both UNIX and Windows utilizing [crossterm](https://github.com/crossterm-rs/crossterm)
- Modularized architecture
  - [promkit-core](https://github.com/ynqa/promkit/tree/v0.10.0/promkit-core/)
    - Core functionality for basic terminal operations and pane management
  - [promkit-widgets](https://github.com/ynqa/promkit/tree/v0.10.0/promkit-widgets/)
    - Various UI components (text, listbox, tree, etc.)
  - [promkit](https://github.com/ynqa/promkit/tree/v0.10.0/promkit)
    - High-level presets and user interfaces
  - [promkit-derive](https://github.com/ynqa/promkit/tree/v0.10.0/promkit-derive/)
    - A Derive macro that simplifies interactive form input
- Rich preset components
  - [Readline](https://github.com/ynqa/promkit/tree/v0.10.0#readline) - Text input with auto-completion
  - [Confirm](https://github.com/ynqa/promkit/tree/v0.10.0#confirm) - Yes/no confirmation prompt
  - [Password](https://github.com/ynqa/promkit/tree/v0.10.0#password) - Password input with masking and validation
  - [Form](https://github.com/ynqa/promkit/tree/v0.10.0#form) - Manage multiple text input fields
  - [Listbox](https://github.com/ynqa/promkit/tree/v0.10.0#listbox) - Single selection interface from a list
  - [QuerySelector](https://github.com/ynqa/promkit/tree/v0.10.0#queryselector) - Searchable selection interface
  - [Checkbox](https://github.com/ynqa/promkit/tree/v0.10.0#checkbox) - Multiple selection checkbox interface
  - [Tree](https://github.com/ynqa/promkit/tree/v0.10.0#tree) - Tree display for hierarchical data like file systems
  - [JSON](https://github.com/ynqa/promkit/tree/v0.10.0#json) - Parse and interactively display JSON data
  - [Text](https://github.com/ynqa/promkit/tree/v0.10.0#text) - Static text display

## Concept

See [here](https://github.com/ynqa/promkit/tree/v0.10.0/Concept.md).

## Projects using *promkit*

- [ynqa/empiriqa](https://github.com/ynqa/empiriqa)
- [ynqa/jnv](https://github.com/ynqa/jnv)
- [ynqa/logu](https://github.com/ynqa/logu)
- [ynqa/sig](https://github.com/ynqa/sig)

## Examples/Demos

*promkit* provides presets so that users can try prompts immediately without
having to build complex components for specific use cases.

Show you commands, code, and actual demo screens for examples
that can be executed immediately below.

### Readline

<details>
<summary>Command</summary>

```bash
cargo run --bin readline --manifest-path examples/readline/Cargo.toml
```

</details>

<details>
<summary>Code</summary>

```rust,ignore
use promkit::{preset::readline::Readline, suggest::Suggest};

fn main() -> anyhow::Result<()> {
    let mut p = Readline::default()
        .title("Hi!")
        .enable_suggest(Suggest::from_iter([
            "apple",
            "applet",
            "application",
            "banana",
        ]))
        .validator(
            |text| text.len() > 10,
            |text| format!("Length must be over 10 but got {}", text.len()),
        )
        .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
```
</details>

<img src="https://github.com/ynqa/promkit/assets/6745370/d124268e-9496-4c4b-83be-c734e4d03591" width="50%" height="auto">

### Confirm

<details>
<summary>Command</summary>

```bash
cargo run --manifest-path examples/confirm/Cargo.toml
```

</details>

<details>
<summary>Code</summary>

```rust,ignore
use promkit::preset::confirm::Confirm;

fn main() -> anyhow::Result<()> {
    let mut p = Confirm::new("Do you have a pet?").prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
```
</details>

<img src="https://github.com/ynqa/promkit/assets/6745370/ac9bac78-66cd-4653-a39f-6c9c0c24131f" width="50%" height="auto">

### Password

<details>
<summary>Command</summary>

```bash
cargo run --manifest-path examples/password/Cargo.toml
```

</details>

<details>
<summary>Code</summary>

```rust,ignore
use promkit::preset::password::Password;

fn main() -> anyhow::Result<()> {
    let mut p = Password::default()
        .title("Put your password")
        .validator(
            |text| 4 < text.len() && text.len() < 10,
            |text| format!("Length must be over 4 and within 10 but got {}", text.len()),
        )
        .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
```
</details>

<img src="https://github.com/ynqa/promkit/assets/6745370/396356ef-47de-44bc-a8d4-d03c7ac66a2f" width="50%" height="auto">

### Form

<details>
<summary>Command</summary>

```bash
cargo run --manifest-path examples/form/Cargo.toml
```

</details>

<details>
<summary>Code</summary>

```rust,ignore
use promkit::{
    crossterm::style::{Color, ContentStyle},
    preset::form::Form,
    promkit_widgets::text_editor,
};

fn main() -> anyhow::Result<()> {
    let mut p = Form::new([
        text_editor::State {
            prefix: String::from("❯❯ "),
            prefix_style: ContentStyle {
                foreground_color: Some(Color::DarkRed),
                ..Default::default()
            },
            active_char_style: ContentStyle {
                background_color: Some(Color::DarkCyan),
                ..Default::default()
            },
            ..Default::default()
        },
        text_editor::State {
            prefix: String::from("❯❯ "),
            prefix_style: ContentStyle {
                foreground_color: Some(Color::DarkGreen),
                ..Default::default()
            },
            active_char_style: ContentStyle {
                background_color: Some(Color::DarkCyan),
                ..Default::default()
            },
            ..Default::default()
        },
        text_editor::State {
            prefix: String::from("❯❯ "),
            prefix_style: ContentStyle {
                foreground_color: Some(Color::DarkBlue),
                ..Default::default()
            },
            active_char_style: ContentStyle {
                background_color: Some(Color::DarkCyan),
                ..Default::default()
            },
            ..Default::default()
        },
    ])
    .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
```

</details>

<img src="https://github.com/ynqa/promkit/assets/6745370/c3dc88a7-d0f0-42f4-90b8-bc4d2e23e36d" width="50%" height="auto">

### Listbox

<details>
<summary>Command</summary>

```bash
cargo run --manifest-path examples/listbox/Cargo.toml
```
</details>

<details>
<summary>Code</summary>

```rust,ignore
use promkit::preset::listbox::Listbox;

fn main() -> anyhow::Result<()> {
    let mut p = Listbox::new(0..100)
        .title("What number do you like?")
        .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
```
</details>

<img src="https://github.com/ynqa/promkit/assets/6745370/0da1b1d0-bb17-4951-8ea8-3b09cd2eb86a" width="50%" height="auto">

### QuerySelector

<details>
<summary>Command</summary>

```bash
cargo run --manifest-path examples/query_selector/Cargo.toml
```
</details>

<details>
<summary>Code</summary>

```rust,ignore
use promkit::preset::query_selector::QuerySelector;

fn main() -> anyhow::Result<()> {
    let mut p = QuerySelector::new(0..100, |text, items| -> Vec<String> {
        text.parse::<usize>()
            .map(|query| {
                items
                    .iter()
                    .filter(|num| query <= num.parse::<usize>().unwrap_or_default())
                    .map(|num| num.to_string())
                    .collect::<Vec<String>>()
            })
            .unwrap_or(items.clone())
    })
    .title("What number do you like?")
    .listbox_lines(5)
    .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
```
</details>

<img src="https://github.com/ynqa/promkit/assets/6745370/7ac2ed54-9f9e-4735-bffb-72f7cee06f6d" width="50%" height="auto">

### Checkbox

<details>
<summary>Command</summary>

```bash
cargo run --manifest-path examples/checkbox/Cargo.toml
```
</details>

<details>
<summary>Code</summary>

```rust,ignore
use promkit::preset::checkbox::Checkbox;

fn main() -> anyhow::Result<()> {
    let mut p = Checkbox::new(vec![
        "Apple",
        "Banana",
        "Orange",
        "Mango",
        "Strawberry",
        "Pineapple",
        "Grape",
        "Watermelon",
        "Kiwi",
        "Pear",
    ])
    .title("What are your favorite fruits?")
    .checkbox_lines(5)
    .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
```
</details>

<img src="https://github.com/ynqa/promkit/assets/6745370/350b16ce-6ef4-46f2-9466-d01b9dab4eaf" width="50%" height="auto">

### Tree

<details>
<summary>Command</summary>

```bash
cargo run --manifest-path examples/tree/Cargo.toml
```
</details>

<details>
<summary>Code</summary>

```rust,ignore
use promkit::{preset::tree::Tree, promkit_widgets::tree::node::Node};

fn main() -> anyhow::Result<()> {
    let mut p = Tree::new(Node::try_from(&std::env::current_dir()?.join("src"))?)
        .title("Select a directory or file")
        .tree_lines(10)
        .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
```
</details>

<img src="https://github.com/ynqa/promkit/assets/6745370/61aefcd0-080a-443e-9dc6-ac627d306f55" width="50%" height="auto">

### JSON

<details>
<summary>Command</summary>

```bash
cargo run --manifest-path examples/json/Cargo.toml
```
</details>

<details>
<summary>Code</summary>

```rust,ignore
use promkit::{
    preset::json::Json,
    promkit_widgets::{
        jsonstream::JsonStream,
        serde_json::{self, Deserializer},
    },
};

fn main() -> anyhow::Result<()> {
    let stream = JsonStream::new(
        Deserializer::from_str(
            r#"
              {
                "apiVersion": "v1",
                "kind": "Pod",
                "metadata": {
                    "annotations": {
                        "kubeadm.kubernetes.io/etcd.advertise-client-urls": "https://172.18.0.2:2379",
                        "kubernetes.io/config.hash": "9c4c3ba79af7ad68d939c568f053bfff",
                        "kubernetes.io/config.mirror": "9c4c3ba79af7ad68d939c568f053bfff",
                        "kubernetes.io/config.seen": "2024-10-12T12:53:27.751706220Z",
                        "kubernetes.io/config.source": "file"
                    },
                    "creationTimestamp": "2024-10-12T12:53:31Z",
                    "labels": {
                        "component": "etcd",
                        "tier": "control-plane"
                    },
                    "name": "etcd-kind-control-plane",
                    "namespace": "kube-system",
                    "ownerReferences": [
                        {
                            "apiVersion": "v1",
                            "controller": true,
                            "kind": "Node",
                            "name": "kind-control-plane",
                            "uid": "6cb2c3e5-1a73-4932-9cc5-6d69b80a9932"
                        }
                    ],
                    "resourceVersion": "192988",
                    "uid": "77465839-5a58-43b1-b754-55deed66d5ca"
                },
                "spec": {
                    "containers": [
                        {
                            "command": [
                                "etcd",
                                "--advertise-client-urls=https://172.18.0.2:2379",
                                "--cert-file=/etc/kubernetes/pki/etcd/server.crt",
                                "--client-cert-auth=true",
                                "--data-dir=/var/lib/etcd",
                                "--experimental-initial-corrupt-check=true",
                                "--experimental-watch-progress-notify-interval=5s",
                                "--initial-advertise-peer-urls=https://172.18.0.2:2380",
                                "--initial-cluster=kind-control-plane=https://172.18.0.2:2380",
                                "--key-file=/etc/kubernetes/pki/etcd/server.key",
                                "--listen-client-urls=https://127.0.0.1:2379,https://172.18.0.2:2379",
                                "--listen-metrics-urls=http://127.0.0.1:2381",
                                "--listen-peer-urls=https://172.18.0.2:2380",
                                "--name=kind-control-plane",
                                "--peer-cert-file=/etc/kubernetes/pki/etcd/peer.crt",
                                "--peer-client-cert-auth=true",
                                "--peer-key-file=/etc/kubernetes/pki/etcd/peer.key",
                                "--peer-trusted-ca-file=/etc/kubernetes/pki/etcd/ca.crt",
                                "--snapshot-count=10000",
                                "--trusted-ca-file=/etc/kubernetes/pki/etcd/ca.crt"
                            ],
                            "image": "registry.k8s.io/etcd:3.5.15-0",
                            "imagePullPolicy": "IfNotPresent",
                            "livenessProbe": {
                                "failureThreshold": 8,
                                "httpGet": {
                                    "host": "127.0.0.1",
                                    "path": "/livez",
                                    "port": 2381,
                                    "scheme": "HTTP"
                                },
                                "initialDelaySeconds": 10,
                                "periodSeconds": 10,
                                "successThreshold": 1,
                                "timeoutSeconds": 15
                            },
                            "name": "etcd",
                            "readinessProbe": {
                                "failureThreshold": 3,
                                "httpGet": {
                                    "host": "127.0.0.1",
                                    "path": "/readyz",
                                    "port": 2381,
                                    "scheme": "HTTP"
                                },
                                "periodSeconds": 1,
                                "successThreshold": 1,
                                "timeoutSeconds": 15
                            },
                            "resources": {
                                "requests": {
                                    "cpu": "100m",
                                    "memory": "100Mi"
                                }
                            },
                            "startupProbe": {
                                "failureThreshold": 24,
                                "httpGet": {
                                    "host": "127.0.0.1",
                                    "path": "/readyz",
                                    "port": 2381,
                                    "scheme": "HTTP"
                                },
                                "initialDelaySeconds": 10,
                                "periodSeconds": 10,
                                "successThreshold": 1,
                                "timeoutSeconds": 15
                            },
                            "terminationMessagePath": "/dev/termination-log",
                            "terminationMessagePolicy": "File",
                            "volumeMounts": [
                                {
                                    "mountPath": "/var/lib/etcd",
                                    "name": "etcd-data"
                                },
                                {
                                    "mountPath": "/etc/kubernetes/pki/etcd",
                                    "name": "etcd-certs"
                                }
                            ]
                        }
                    ],
                    "dnsPolicy": "ClusterFirst",
                    "enableServiceLinks": true,
                    "hostNetwork": true,
                    "nodeName": "kind-control-plane",
                    "preemptionPolicy": "PreemptLowerPriority",
                    "priority": 2000001000,
                    "priorityClassName": "system-node-critical",
                    "restartPolicy": "Always",
                    "schedulerName": "default-scheduler",
                    "securityContext": {
                        "seccompProfile": {
                            "type": "RuntimeDefault"
                        }
                    },
                    "terminationGracePeriodSeconds": 30,
                    "tolerations": [
                        {
                            "effect": "NoExecute",
                            "operator": "Exists"
                        }
                    ],
                    "volumes": [
                        {
                            "hostPath": {
                                "path": "/etc/kubernetes/pki/etcd",
                                "type": "DirectoryOrCreate"
                            },
                            "name": "etcd-certs"
                        },
                        {
                            "hostPath": {
                                "path": "/var/lib/etcd",
                                "type": "DirectoryOrCreate"
                            },
                            "name": "etcd-data"
                        }
                    ]
                },
                "status": {
                    "conditions": [
                        {
                            "lastProbeTime": null,
                            "lastTransitionTime": "2024-12-06T13:28:35Z",
                            "status": "True",
                            "type": "PodReadyToStartContainers"
                        },
                        {
                            "lastProbeTime": null,
                            "lastTransitionTime": "2024-12-06T13:28:34Z",
                            "status": "True",
                            "type": "Initialized"
                        },
                        {
                            "lastProbeTime": null,
                            "lastTransitionTime": "2024-12-06T13:28:50Z",
                            "status": "True",
                            "type": "Ready"
                        },
                        {
                            "lastProbeTime": null,
                            "lastTransitionTime": "2024-12-06T13:28:50Z",
                            "status": "True",
                            "type": "ContainersReady"
                        },
                        {
                            "lastProbeTime": null,
                            "lastTransitionTime": "2024-12-06T13:28:34Z",
                            "status": "True",
                            "type": "PodScheduled"
                        }
                    ],
                    "containerStatuses": [
                        {
                            "containerID": "containerd://de0d57479a3ac10e213df6ea4fc1d648ad4d70d4ddf1b95a7999d0050171a41e",
                            "image": "registry.k8s.io/etcd:3.5.15-0",
                            "imageID": "sha256:27e3830e1402783674d8b594038967deea9d51f0d91b34c93c8f39d2f68af7da",
                            "lastState": {
                                "terminated": {
                                    "containerID": "containerd://28d1a65bd9cfa40624a0c17979208f66a5cc7f496a57fa9a879907bb936f57b3",
                                    "exitCode": 255,
                                    "finishedAt": "2024-12-06T13:28:31Z",
                                    "reason": "Unknown",
                                    "startedAt": "2024-11-04T15:14:19Z"
                                }
                            },
                            "name": "etcd",
                            "ready": true,
                            "restartCount": 2,
                            "started": true,
                            "state": {
                                "running": {
                                    "startedAt": "2024-12-06T13:28:35Z"
                                }
                            }
                        }
                    ],
                    "hostIP": "172.18.0.2",
                    "hostIPs": [
                        {
                            "ip": "172.18.0.2"
                        }
                    ],
                    "phase": "Running",
                    "podIP": "172.18.0.2",
                    "podIPs": [
                        {
                            "ip": "172.18.0.2"
                        }
                    ],
                    "qosClass": "Burstable",
                    "startTime": "2024-12-06T13:28:34Z"
                }
              }
            "#,
        )
        .into_iter::<serde_json::Value>()
        .filter_map(serde_json::Result::ok)
        .collect::<Vec<_>>()
        .iter(),
    );

    let mut p = Json::new(stream).title("JSON viewer").prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
```
</details>

<img src="https://github.com/ynqa/promkit/assets/6745370/751af3ae-5aff-45ca-8729-34cd004ee7d9" width="50%" height="auto">

## License

This project is licensed under the MIT License.
See the [LICENSE](https://github.com/ynqa/promkit/blob/main/LICENSE)
file for details.

## Stargazers over time
[![Stargazers over time](https://starchart.cc/ynqa/promkit.svg?variant=adaptive)](https://starchart.cc/ynqa/promkit)
