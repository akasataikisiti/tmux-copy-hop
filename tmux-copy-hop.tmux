#!/usr/bin/env bash

tmux bind-key j run-shell "tmux-copy-hop jump"
tmux bind-key l run-shell "tmux-copy-hop line-jump"
