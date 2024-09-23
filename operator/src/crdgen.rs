use kube::CustomResourceExt;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        if args[1] == "json" {
            print!(
                "{}",
                serde_json::to_string_pretty(&operator::KupoPort::crd()).unwrap()
            );
            return;
        }
    }

    print!(
        "{}",
        serde_yaml::to_string(&operator::KupoPort::crd()).unwrap()
    )
}
