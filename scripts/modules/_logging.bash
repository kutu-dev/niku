#!/usr/bin/env bash
set -euo pipefail
shopt -s globstar

COLOR_FORE_GREEN="$(tput setaf 2)"
COLOR_FORE_WHITE="$(tput setaf 7)"
COLOR_FORE_RED="$(tput setaf 1)"

COLOR_STYLE_BOLD=$(tput bold)
COLOR_STYLE_NORMAL=$(tput sgr0)

# $1 -> Log level color code
# $2 -> Log level text
# $3 -> Message
_colored_log() {
  echo "$COLOR_STYLE_BOLD$1$2$COLOR_FORE_WHITE:$COLOR_STYLE_NORMAL $3"
}

info() {
  _colored_log "$COLOR_FORE_GREEN" INFO "$*"
}

error() {
  _colored_log "$COLOR_FORE_RED" ERROR "$*"
}
