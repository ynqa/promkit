use promkit::{
    preset::json::Json,
    widgets::{
        jsonstream::JsonStream,
        serde_json::{self, Deserializer},
    },
    Prompt,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    Json::new_with_json(stream).title("JSON viewer").run().await
}
