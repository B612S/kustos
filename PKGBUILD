# Maintainer: B612S <sem.alimov@inbox.ru>
pkgname=kustos
pkgver=0.2.0
pkgrel=1
pkgdesc="Time Machine-like encrypted, compressed backups with a Kirigami GUI"
arch=('x86_64')
url="https://github.com/B612S/kustos"
license=('GPL-3.0-or-later')
depends=('qt6-base' 'qt6-declarative' 'kirigami' 'qqc2-desktop-style' 'rustic' 'xdg-utils')
makedepends=('rust' 'pkgconf')
optdepends=('libnotify: desktop notifications when a backup finishes')
options=(!lto)
source=()
sha256sums=()

build() {
  cd "$startdir"
  export QMAKE=/usr/bin/qmake6
  cargo build --release --locked --target-dir "$srcdir/target"
}

check() {
  cd "$startdir"
  export QMAKE=/usr/bin/qmake6
  cargo test --locked --target-dir "$srcdir/target"
}

package() {
  install -Dm755 "$srcdir/target/release/kustos" "$pkgdir/usr/bin/kustos"
  install -Dm644 "$startdir/kustos.desktop" \
    "$pkgdir/usr/share/applications/kustos.desktop"
  install -Dm644 "$startdir/assets/kustos.svg" \
    "$pkgdir/usr/share/icons/hicolor/scalable/apps/kustos.svg"
  install -Dm644 "$startdir/assets/kustos.png" \
    "$pkgdir/usr/share/icons/hicolor/256x256/apps/kustos.png"
  install -Dm644 "$startdir/LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  install -Dm644 "$startdir/docs/readme/ru.md" \
    "$pkgdir/usr/share/doc/$pkgname/README.ru.md"
  install -Dm644 "$startdir/docs/BACKUP-FORMAT.md" \
    "$pkgdir/usr/share/doc/$pkgname/BACKUP-FORMAT.md"
}
