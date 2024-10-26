#!/bin/bash
#
# prompt.sh - Hook into promptpath for Bash and Zsh
#
# Usage:
#   Source this file in your .bashrc or .zshrc:
#   source /path/to/prompt.sh
#
# Update project mappings in ~/.config/promptpath/config.toml
#
[ ${ZSH_VERSION} ] && precmd() { prompt; }
[ ${BASH_VERSION} ] && PROMPT_COMMAND=prompt

prompt() {
  pwddisplay=$PWD
  if command -v promptpath &> /dev/null; then
    pwddisplay=$(promptpath)
  fi
  if [ ${ZSH_VERSION} ]; then
    PROMPT="%F{$GREEN}$pwddisplay%f%F{$BLUE}~>%f "
  elif [ ${BASH_VERSION} ]; then
    PS1='\[\e[32m\]$pwddisplay\[\e[m\]\[\e[32m\]~>\[\e[m\] '
  fi
}