#!/usr/bin/env bash
# Pin AGP-compatible JDK (17/21). JDK 26+ makes AGP 8 throw a misleading
# "compileSdkVersion is not specified" while parsing the Java version.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
if [[ -z "${JAVA_HOME:-}" ]] || ! "$JAVA_HOME/bin/java" -version 2>&1 | grep -Eq 'version "(17|21)\.'; then
  if command -v /usr/libexec/java_home >/dev/null 2>&1; then
    JAVA_HOME="$(/usr/libexec/java_home -v 17 2>/dev/null || /usr/libexec/java_home -v 21 2>/dev/null || true)"
  fi
fi
if [[ -z "${JAVA_HOME:-}" || ! -x "${JAVA_HOME}/bin/java" ]]; then
  echo "test:android needs JDK 17 or 21 (AGP 8). Set JAVA_HOME." >&2
  exit 1
fi
export JAVA_HOME
cd "$ROOT/android"
exec ./gradlew "$@"
