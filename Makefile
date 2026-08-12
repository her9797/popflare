VERSION ?= $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
TAG := v$(VERSION)
APP_NAME := Popflare
ZIP_NAME := $(APP_NAME)-$(TAG)-macos-arm64.zip
DIST_DIR := dist
APP_PATH := $(DIST_DIR)/$(APP_NAME).app
ZIP_PATH := $(DIST_DIR)/$(ZIP_NAME)

.PHONY: help check test app sign verify package release clean open

help:
	@echo "Popflare commands"
	@echo "  make check                 cargo check"
	@echo "  make test                  cargo test"
	@echo "  make app                   build dist/Popflare.app"
	@echo "  make sign                  adhoc sign dist/Popflare.app"
	@echo "  make verify                verify app signature"
	@echo "  make package                build, sign, zip using Cargo.toml version"
	@echo "  make release                package, tag, push, and create GitHub Release"
	@echo "  make open                  open dist/Popflare.app"
	@echo "  make clean                 remove dist"

check:
	cargo check

test:
	cargo test

app:
	./scripts/build-app.sh

sign: app
	codesign --force --deep --sign - $(APP_PATH)

verify: sign
	codesign --verify --deep --strict --verbose=2 $(APP_PATH)

package: test verify
	cd $(DIST_DIR) && rm -f $(ZIP_NAME) && zip -r $(ZIP_NAME) $(APP_NAME).app
	@echo "Packaged $(ZIP_PATH)"

release: package
	git diff --quiet
	git diff --cached --quiet
	@if git rev-parse --verify $(TAG) >/dev/null 2>&1; then echo "Tag $(TAG) already exists. Bump Cargo.toml version or run make release VERSION=x.y.z."; exit 1; fi
	git tag $(TAG)
	git push origin main
	git push origin $(TAG)
	printf "%b" "Popflare $(TAG) macOS release.\n\n- 메뉴막대 상주 앱\n- 클릭 이펙트 on/off\n- Color Burst, Rainbow Circle, Pink Sparkles, Color Sparkles 옵션\n- macOS arm64용 Popflare.app zip 포함\n\n참고: 개인용 adhoc 서명이라 다른 Mac에서는 최초 실행 시 확인되지 않은 개발자 경고가 뜰 수 있습니다. Finder에서 앱을 우클릭한 뒤 열기를 선택하세요.\n" > $(DIST_DIR)/release-notes-$(TAG).md
	gh release create $(TAG) $(ZIP_PATH) --title "Popflare $(TAG)" --notes-file $(DIST_DIR)/release-notes-$(TAG).md

open: app
	open $(APP_PATH)

clean:
	rm -rf $(DIST_DIR)
