# Route 53 Dynamic DNS Client

This is a client that interacts with the Route 53 API using the AWS SDK for Rust
to dynamically update a DNS record.

## Getting Started

### Setting up IAM and Notifications

Since this project interacts with the Route 53 API directly, it will be necessary to
have an IAM principal with permissions to invoke `route53:ChangeResourceRecordSets`
on the hosted zone where the dynamic DNS record will be. This is the only required
permission. Conditions can be used to limit the actions that can be performed on
the resource record set.

There is an included sample CloudFormation template,
`user-and-notification.template.yml` which creates the IAM user as well as the
necessary infrastructure to send an email every time that user updates the given
Route 53 Hosted Zone. The notifications are based on EventBridge and SNS with a
Lambda function to transform the event into a more useful message. It will be
necessary to create the Access Key for the user manually. The template also uses
the [fine-grained permissions][rrset-conditions] to limit to UPSERT operations on
A and AAAA records.

[rrset-conditions]: https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/specifying-rrset-conditions.html

### Building the project

The project can be build with `cargo build`, optionally with `cargo build --release`.

### Running the project

A configuration file based on `sample_config.yml` needs to be created. This should
provide the specific values defined in the sample. Then, export your AWS credentials
using the standard environment variable names.

You can then run the command with:

```
$ ./target/debug/dynamic-route53 -c <FILE>
```

which should properly update the entry in Route 53.

### On Arch Linux

A (less-than-great) PKGBUILD is provided that will put the systemd files in place
to run this on a timer. Install the package using the PKGBUILD as you usually would.
Update the configuration files in `/etc/dynamic-route53` and then enable the timer with

```
# systemctl enable dynamic-route53.timer
```

### As a container

A `Containerfile` is included in the repo. That is automatically pushed to
[`ghcr.io/kylelaker/dynamic-route53`](https://ghcr.io/kylelaker/dynamic-route53). Additionally,
a helm chart is included in `helm/dynamic-route53` that can be used for deploying within Kubernetes.

The `CONFIG_FILE`, `AWS_ACCESS_KEY_ID`, and `AWS_SECRET_ACCESS_KEY` environment variables should
be provided to a created container (the Helm chart manages this).

## To Do

- [ ] Handle errors more correctly
- [ ] Support IPv6/AAAA
- [ ] Improve systemd configs
- [ ] Consider actually daemonizing rather than using a `systemd.timer(5)`
- [ ] Docs
