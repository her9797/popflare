# popflare

macOS에서 마우스를 클릭할 때마다 클릭 위치에 작은 폭죽 이펙트를 터뜨리는 장난감 앱입니다.

메뉴막대에 상주하면서 동작하고, 클릭 이벤트는 가로채지 않습니다. 즉, 평소처럼 Mac을 사용하면서 클릭할 때만 화면 위에 파티클 효과가 잠깐 나타납니다.

## 기능

- 전역 마우스 클릭 감지
- 투명 오버레이 창 위에 폭죽 파티클 렌더링
- 실제 클릭은 기존 앱으로 그대로 전달
- 메뉴막대 아이콘 표시
- 메뉴에서 이펙트 켜기/끄기
- 메뉴에서 앱 종료
- 배포용 `Popflare.app` 번들 생성 스크립트 제공

## 개발 실행

```bash
cargo run
```

실행 후 메뉴막대에 popflare 아이콘이 뜹니다.

메뉴에서 할 수 있는 것:

```txt
Enabled        이펙트 켜기/끄기
Quit Popflare  종료
```

## 앱 빌드

배포용 `.app`을 만들려면:

```bash
./scripts/build-app.sh
```

빌드 결과:

```txt
dist/Popflare.app
```

실행:

```bash
open dist/Popflare.app
```

다른 Mac에 옮길 때는 Finder에서 `Popflare.app` 하나만 복사하면 됩니다. 터미널에서는 `.app`이 폴더처럼 보이지만, macOS에서는 하나의 앱 파일처럼 표시됩니다.


## 처음 실행할 때

Popflare는 아직 Apple Developer ID 서명과 notarization을 하지 않은 개인 빌드입니다.  
그래서 GitHub Release에서 zip을 내려받아 처음 실행하면 macOS가 “확인되지 않은 개발자” 또는 “악성코드 여부를 확인할 수 없음” 같은 경고를 띄울 수 있습니다.

압축을 푼 뒤 `Popflare.app`이 다운로드 폴더에 있다면, 최초 1회만 아래 명령어를 실행하세요.

```bash
xattr -dr com.apple.quarantine ~/Downloads/Popflare.app
chmod +x ~/Downloads/Popflare.app/Contents/MacOS/popflare
open ~/Downloads/Popflare.app
```

한 번 실행한 뒤에는 같은 `Popflare.app`에 대해 다시 할 필요가 없습니다.  
다만 새 버전을 다시 다운로드하거나 다른 Mac으로 옮긴 경우에는 한 번 더 필요할 수 있습니다.

## 주의사항

- 현재는 macOS 전용입니다.
- 서명/notarization은 아직 하지 않았습니다.
- 다른 Mac에서 처음 실행할 때 “확인되지 않은 개발자” 경고가 뜰 수 있습니다.
- Apple Silicon에서 빌드한 앱은 기본적으로 Apple Silicon Mac용입니다.
- Intel Mac까지 지원하려면 universal binary 빌드가 필요합니다.
- macOS 설정에 따라 전역 클릭 감지를 위해 접근성 권한이 필요할 수 있습니다.

## 구조

```txt
src/effect.rs            파티클 이펙트 엔진
src/platform/macos.rs    macOS 메뉴막대, 오버레이, 클릭 감지
assets/                  메뉴막대 아이콘
scripts/build-app.sh     Popflare.app 빌드 스크립트
```

## 목표

쓸모 있는 척이라기보다, 켜두면 그냥 기분 좋아지는 작은 macOS 장난감 앱을 목표로 합니다.
