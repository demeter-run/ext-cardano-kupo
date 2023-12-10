use kube::CustomResourceExt;

fn main() {
    print!(
        "{}",
        serde_yaml::to_string(&controller::KupoPort::crd()).unwrap()
    )
}
