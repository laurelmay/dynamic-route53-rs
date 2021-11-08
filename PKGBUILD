# Maintainer: Kyle Laker <kyle@laker.email>

_pkgbase=dynamic-route53
pkgname="${_pkgbase}"-git
pkgver=0.1.0
pkgrel=1
pkgdesc="Dynamically update Route 53 with the current IP"
arch=("x86_64")
url="https://github.com/kylelaker/dynamic-route53-rs"
license=(MIT)
depends=()
makedepends=(git rust)
source=("${pkgname}::git+${url}")
sha512sums=('SKIP')
backup=(etc/dynamic-route53/config.yml
        etc/dynamic-route53/aws-vars
)

pkgver() {
  cd "$pkgname"
  printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
}

build() {
  cd "$srcdir/$pkgname"
  cargo build --release --locked --all-features --target-dir=target
}

check() {
  cd "$srcdir/$pkgname"
  cargo test --release --locked --target-dir=target
}

package() {
  cd "$srcdir/$pkgname"
  install -Dm 755 "target/release/${_pkgbase}" -t "${pkgdir}/usr/bin/"

  # Systemd Units
  mkdir -p "${pkgdir}/usr/lib/systemd/system/"
  install -Dm 644 "systemd/dynamic-route53.service" "${pkgdir}/usr/lib/systemd/system/"
  install -Dm 644 "systemd/dynamic-route53.timer" "${pkgdir}/usr/lib/systemd/system/"

  # Configuaration files
  mkdir -p "${pkgdir}/etc/dynamic-route53/"
  install -Dm 644 "sample_config.yml" "${pkgdir}/etc/dynamic-route53/config.yml"
  touch "${pkgdir}/etc/dynamic-route53/aws-vars"

  install -Dm 644 LICENSE "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"
}
